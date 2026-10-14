import { test, expect, type Page } from '@playwright/test'
import { execFileSync, spawn, type ChildProcess } from 'node:child_process'
import { createServer } from 'node:net'
import { mkdtemp, rm } from 'node:fs/promises'
import os from 'node:os'
import path from 'node:path'
import { setTimeout as delay } from 'node:timers/promises'

const root = process.cwd()
const daemonPath = path.join(root, 'target', 'debug', `command-scheduler${process.platform === 'win32' ? '.exe' : ''}`)
let daemon: ChildProcess | undefined
let tempDir = ''
let baseUrl = ''

async function seedReviewSchedules(page: Page) {
  const schedules: Array<{id: string; name: string}> = []
  for (const name of ['review-A', 'review-B']) {
    const response = await page.request.post(`${baseUrl}/api/v1/schedules`, { data: {
      name, program: process.execPath, args: ['-e', `console.log('${name}-stdout')`],
      timing: {type:'once', at:'2099-01-01T00:00:00Z'}, missedPolicy:'skip', retryPolicy:{maxRetries:0},
    } })
    expect(response.status()).toBe(201)
    const schedule = await response.json() as {id:string;name:string}
    const run = await (await page.request.post(`${baseUrl}/api/v1/schedules/${schedule.id}/runs`)).json() as {id:string}
    await expect.poll(async () => (await (await page.request.get(`${baseUrl}/api/v1/occurrences/${run.id}`)).json() as {status:string}).status).toBe('succeeded')
    schedules.push(schedule)
  }
  return schedules
}

test('discards a late history response after selecting another schedule', async ({ page }) => {
  const [a] = await seedReviewSchedules(page)
  await page.clock.install()
  await page.goto(baseUrl)
  let release!: () => void
  let started!: () => void
  const pending = new Promise<void>(resolve => { started=resolve })
  const hold = new Promise<void>(resolve => { release=resolve })
  await page.route(`**/api/v1/schedules/${a.id}/occurrences?*`, async route => {
    const response=await route.fetch(); started(); await hold; await route.fulfill({response})
  })
  await page.locator('.schedule-card').filter({hasText:'review-A'}).locator('.schedule-summary').click()
  await pending
  await page.locator('.schedule-card').filter({hasText:'review-B'}).locator('.schedule-summary').click()
  const details=page.locator('.schedule-details')
  await expect(details.locator('pre')).toHaveText('review-B-stdout\n')
  release()
  await expect(details.locator('.occurrence-row')).toHaveCount(1)
  await expect(details.locator('pre')).toHaveText('review-B-stdout\n')
})

test('discards a pending automatic refresh after selection or filter changes', async ({ page }) => {
  const [a] = await seedReviewSchedules(page)
  await page.clock.install()
  await page.goto(baseUrl)
  await page.locator('.schedule-card').filter({hasText:'review-A'}).locator('.schedule-summary').click()
  await expect(page.locator('.schedule-details pre')).toHaveText('review-A-stdout\n')
  for (const change of ['selection', 'filter']) {
    let release!: () => void
    let started!: () => void
    let completed!: () => void
    const pending=new Promise<void>(resolve => { started=resolve })
    const hold=new Promise<void>(resolve => { release=resolve })
    const done=new Promise<void>(resolve => { completed=resolve })
    await page.route(`**/api/v1/schedules/${a.id}/occurrences?*`, async route => {
      const response=await route.fetch(); started(); await hold; await route.fulfill({response}); completed()
    }, {times:1})
    await page.clock.runFor(3000)
    await pending
    if(change==='selection') {
      await page.locator('.schedule-card').filter({hasText:'review-B'}).locator('.schedule-summary').click()
      await expect(page.locator('.schedule-details pre')).toHaveText('review-B-stdout\n')
    } else {
      await page.locator('.schedule-details').getByLabel('Estado').selectOption('failed')
      await expect(page.locator('.schedule-details .occurrence-row')).toHaveCount(0)
    }
    release(); await done
    // Wait for the response's continuation, including automatic-refresh merging.
    await page.clock.runFor(50)
    if(change==='selection') {
      await expect(page.locator('.schedule-details .occurrence-row')).toHaveCount(1)
      await expect(page.locator('.schedule-details pre')).toHaveText('review-B-stdout\n')
      await page.locator('.schedule-card').filter({hasText:'review-A'}).locator('.schedule-summary').click()
      await expect(page.locator('.schedule-details pre')).toHaveText('review-A-stdout\n')
    } else {
      await expect(page.locator('.schedule-details .occurrence-row')).toHaveCount(0)
    }
  }
})

async function unusedPort() {
  const server = createServer()
  await new Promise<void>(resolve => server.listen(0, '127.0.0.1', resolve))
  const address = server.address()
  if (!address || typeof address === 'string') throw new Error('Could not select an ephemeral port')
  await new Promise<void>((resolve, reject) => server.close(error => error ? reject(error) : resolve()))
  return address.port
}

async function waitForDaemon() {
  const deadline = Date.now() + 15_000
  while (Date.now() < deadline) {
    if (daemon?.exitCode !== null && daemon?.exitCode !== undefined) throw new Error(`Daemon exited with ${daemon.exitCode}`)
    try {
      const response = await fetch(`${baseUrl}/health`)
      if (response.ok && await response.text() === 'ok') return
    } catch { /* daemon is still starting */ }
    await delay(100)
  }
  throw new Error('Daemon did not become healthy')
}

async function createSchedule(page: Page, name: string, program = process.execPath, args = '["-e","console.log(\'ui-ok\')"]', maxRetries?: number) {
  await page.getByRole('button', { name: '＋ Novo schedule' }).click()
  await page.getByLabel(/Nome/).fill(name)
  await page.getByLabel(/Executável/).fill(program)
  await page.getByLabel(/Argumentos/).fill(args)
  await page.getByLabel('Repetição').selectOption('advanced')
  await page.getByLabel(/Cron de cinco campos/).fill('0 0 1 1 *')
  if (maxRetries !== undefined) {
    await page.getByText('Opções de execução').click()
    await page.getByLabel('Tentativas adicionais').fill(String(maxRetries))
  }
  await page.locator('.form-panel').getByRole('button', { name: 'Criar schedule', exact: true }).click()
  await expect(page.getByRole('status')).toContainText('Schedule criado.')
  await expect(page.locator('.schedule-card').getByText(name, { exact: true })).toBeVisible()
}

test.beforeEach(async () => {
  tempDir = await mkdtemp(path.join(os.tmpdir(), 'command-scheduler-web-'))
  const port = await unusedPort()
  baseUrl = `http://127.0.0.1:${port}`
  daemon = spawn(daemonPath, [], {
    cwd: root,
    stdio: 'ignore',
    env: {
      ...process.env,
      COMMAND_SCHEDULER_ADDR: `127.0.0.1:${port}`,
      COMMAND_SCHEDULER_DB_PATH: path.join(tempDir, 'web.sqlite3'),
      COMMAND_SCHEDULER_WEB_DIR: path.join(root, 'ui', 'dist'),
    },
  })
  await waitForDaemon()
})

test.afterEach(async () => {
  if (daemon && daemon.exitCode === null) {
    const exited = new Promise<void>(resolve => daemon?.once('exit', () => resolve()))
    daemon.kill('SIGTERM')
    await Promise.race([exited, delay(5_000)])
    if (daemon.exitCode === null) daemon.kill('SIGKILL')
  }
  daemon = undefined
  if (tempDir) await rm(tempDir, { recursive: true, force: true })
})

// Covers: BDD-JRN-001-01 BDD-JRN-001-11 BDD-JRN-001-18
 test('creates a valid schedule with JSON parameters and displays the run output', async ({ page }) => {
  await page.goto(baseUrl)
  await createSchedule(page, 'ui-parameters', process.execPath, '["-e","console.log(process.env.COMMAND_SCHEDULER_PARAMS_JSON)"]')
  await page.getByLabel(/Parâmetros atuais/).fill('{"origin":"web"}')
  await page.getByRole('button', { name: 'Atualizar parâmetros' }).click()
  await expect(page.getByRole('status')).toContainText('Parâmetros atualizados.')
  await page.getByRole('button', { name: 'Executar agora' }).click()
  const row = page.locator('.occurrence-row').first()
  await expect(row).toContainText('Concluído', { timeout: 10_000 })
  await row.getByText('Ver saída').click()
  await expect(row.locator('pre').first()).toContainText('{"origin":"web"}')
})

// Covers: BDD-JRN-001-17
test('creates a recurring schedule from a local calendar preset', async ({ page }) => {
  await page.goto(baseUrl)
  await page.getByRole('button', { name: '＋ Novo schedule' }).click()
  await page.getByLabel(/Nome/).fill('preset-daily')
  await page.getByLabel(/Executável/).fill(process.execPath)
  await page.getByLabel(/Argumentos/).fill('[]')
  await page.getByLabel('Repetição').selectOption('daily')
  await page.getByLabel('Horário local').fill('09:30')
  const createResponse = page.waitForResponse(response => response.url().includes('/api/v1/schedules') && response.request().method() === 'POST')
  await page.locator('.form-panel').getByRole('button', { name: 'Criar schedule', exact: true }).click()
  const response = await createResponse
  expect((await response.json()).timing.expression).toBe('30 9 * * *')
  await expect(page.getByText('preset-daily', { exact: true })).toBeVisible()
})

// Covers: BDD-JRN-001-15
test('tests the command and displays output without creating a schedule or occurrence', async ({ page }) => {
  await page.goto(baseUrl)
  await page.getByRole('button', { name: '＋ Novo schedule' }).click()
  await page.getByLabel(/Executável/).fill(process.execPath)
  await page.getByLabel(/Argumentos/).fill('["-e","console.log(\'test-output\')"]')
  await page.getByRole('button', { name: /Testar comando/ }).click()
  const result = page.locator('.command-test-result')
  await expect(result).toContainText('código 0')
  await expect(result.locator('pre').first()).toContainText('test-output')
  const schedules = await (await fetch(`${baseUrl}/api/v1/schedules?limit=100`)).json() as {items: unknown[]}
  const runs = await (await fetch(`${baseUrl}/api/v1/occurrences?limit=100`)).json() as {items: unknown[]}
  expect(schedules.items).toHaveLength(0)
  expect(runs.items).toHaveLength(0)
})

// Additional command-test error coverage.
test('shows a failed command test when the executable cannot start', async ({ page }) => {
  await page.goto(baseUrl)
  await page.getByRole('button', { name: '＋ Novo schedule' }).click()
  await page.getByLabel(/Executável/).fill(path.join(tempDir, 'missing-executable'))
  await page.getByLabel(/Argumentos/).fill('[]')
  await page.getByRole('button', { name: /Testar comando/ }).click()
  const result = page.locator('.command-test-result')
  await expect(result).toContainText('não iniciou')
  await expect(result.locator('pre').first()).toContainText('could not start')
  const schedules = await (await fetch(`${baseUrl}/api/v1/schedules?limit=100`)).json() as {items: unknown[]}
  const runs = await (await fetch(`${baseUrl}/api/v1/occurrences?limit=100`)).json() as {items: unknown[]}
  expect(schedules.items).toHaveLength(0)
  expect(runs.items).toHaveLength(0)
})

// Covers: BDD-JRN-001-16
test('rejects a complete command line pasted into the executable field', async ({ page }) => {
  await page.goto(baseUrl)
  await page.getByRole('button', { name: '＋ Novo schedule' }).click()
  await page.getByLabel(/Executável/).fill(`${process.execPath} -e console.log\('not-run'\)`)
  await page.getByLabel(/Argumentos/).fill('[]')
  await expect(page.getByRole('alert')).toContainText('Parece que você colou a linha de comando inteira')
  await expect(page.getByRole('button', { name: /Testar comando/ })).toBeDisabled()
  await expect(page.locator('.form-panel').getByRole('button', { name: 'Criar schedule', exact: true })).toBeDisabled()
  const schedules = await (await fetch(`${baseUrl}/api/v1/schedules?limit=100`)).json() as {items: unknown[]}
  const runs = await (await fetch(`${baseUrl}/api/v1/occurrences?limit=100`)).json() as {items: unknown[]}
  expect(schedules.items).toHaveLength(0)
  expect(runs.items).toHaveLength(0)
})

// Covers: BDD-JRN-016-01 BDD-JRN-016-02 BDD-JRN-016-03 BDD-JRN-016-06
test('shows cockpit metrics and switches the agenda calendar range', async ({ page }) => {
  await page.goto(baseUrl)
  await expect(page).toHaveTitle('commandscheduler')
  await expect(page.getByText('Daemon local', { exact: true })).toBeVisible()
  await expect(page.getByText('Nenhum agendamento ativo')).toBeVisible()
  await expect(page.getByText('FALHAS · PERÍODO').locator('..')).toContainText('0')
  await page.getByLabel('Indicadores').selectOption('7')
  for (const range of ['day', 'week', 'month']) {
    await page.getByLabel('Visualização').selectOption(range)
    await expect(page.getByRole('heading', { name: 'Agendamentos e execuções' })).toBeVisible()
  }
  await page.getByRole('button', { name: '＋ Novo schedule' }).click()
  await page.getByLabel(/Nome/).fill('agenda-target')
  await page.getByLabel(/Executável/).fill(process.execPath)
  await page.getByLabel(/Argumentos/).fill('[]')
  await page.getByLabel('Repetição').selectOption('minutes')
  await page.locator('.form-panel fieldset select').nth(1).selectOption('1')
  await page.locator('.form-panel').getByRole('button', { name: 'Criar schedule', exact: true }).click()
  await expect(page.locator('.agenda-list').getByRole('button', { name: 'agenda-target' }).first()).toBeVisible()
  await page.locator('.agenda-list').getByRole('button', { name: 'agenda-target' }).first().click()
  await expect(page.getByRole('button', { name: 'Executar agora' })).toBeVisible()
  const scheduleResponse = await fetch(`${baseUrl}/api/v1/schedules?limit=100`)
  const schedulePage = await scheduleResponse.json() as {items:Array<{id:string;name:string}>}
  const scheduleId = schedulePage.items.find(item=>item.name==='agenda-target')?.id
  expect(scheduleId).toBeTruthy()
  const python = process.platform === 'win32' ? 'python' : 'python3'
  const insertOutcomes = 'import sqlite3,sys,time,uuid; db,sid=sys.argv[1:]; now=int(time.time()*1000000); c=sqlite3.connect(db); c.executemany("INSERT INTO occurrences(id,schedule_id,status,source,created_at,finished_at) VALUES(?,? ,?,\'manual\',?,?)", [(str(uuid.uuid4()),sid,status,now+i,now+i) for i,status in enumerate([\'succeeded\',\'failed\'])]); c.commit()'
  execFileSync(python,['-c',insertOutcomes,path.join(tempDir,'web.sqlite3'),scheduleId!])
  await page.getByRole('button',{name:'Atualizar ↻'}).click()
  await expect(page.locator('.metric-grid article').nth(1).locator('strong')).toHaveText('1')
  await expect(page.locator('.agenda-list')).toContainText('Concluído')
  await expect(page.locator('.agenda-list')).toContainText('Falhou')
  await expect(page.locator('.metric-grid article').nth(4).locator('strong')).toHaveText('1')
  await expect(page.locator('.metric-grid article').nth(5).locator('strong')).toHaveText('1')
  for(const days of ['1','7','30','90']) { await page.getByLabel('Indicadores').selectOption(days); await expect(page.locator('.metric-grid article').nth(4).locator('strong')).toHaveText('1') }
})

// Covers: BDD-JRN-016-07
test('marks daemon health unavailable and clears stale cockpit numbers', async ({ page }) => {
  await page.route('**/health', route => route.fulfill({ status: 503, body: 'unavailable' }))
  await page.goto(baseUrl)
  await expect(page.getByText('Daemon indisponível')).toBeVisible()
  await expect(page.getByText('Nenhum agendamento ativo')).toBeVisible()
  await expect(page.locator('.metric-grid')).toContainText('—')
})

// Covers: BDD-JRN-001-02
 test('rejects an invalid schedule without adding it to the list', async ({ page }) => {
  await page.goto(baseUrl)
  await page.getByRole('button', { name: '＋ Novo schedule' }).click()
  await page.getByLabel(/Executável/).fill(process.execPath)
  await page.getByLabel(/Argumentos/).fill('[]')
  await page.getByLabel('Repetição').selectOption('advanced')
  await page.getByLabel(/Cron de cinco campos/).fill('not-a-cron')
  await page.locator('.form-panel').getByRole('button', { name: 'Criar schedule', exact: true }).click()
  await expect(page.getByRole('alert')).toBeVisible()
  await expect(page.getByRole('heading', { name: 'Nenhum schedule ainda' })).toBeVisible()
})

// Covers: BDD-JRN-005-01 BDD-JRN-005-02 BDD-JRN-005-03 BDD-JRN-005-05 BDD-JRN-016-04
 test('edits, pauses, resumes and cancels a schedule from the web interface', async ({ page }) => {
  await page.goto(baseUrl)
  await createSchedule(page, 'ui-lifecycle')
  await page.getByRole('button', { name: 'Editar' }).click()
  const edit = page.locator('form.inline-form').first()
  await edit.getByLabel('Nome').fill('ui-lifecycle-edited')
  await edit.getByLabel('Política de vencidas').selectOption('sequence')
  await edit.getByRole('button', { name: 'Salvar alteração' }).click()
  await expect(page.locator('.schedule-card').getByText('ui-lifecycle-edited', { exact: true })).toBeVisible()
  await page.getByRole('button', { name: /Pausar/ }).click()
  await expect(page.getByText('Pausado', { exact: true })).toBeVisible()
  await page.getByRole('button', { name: /Retomar/ }).click()
  await expect(page.getByText('Ativo', { exact: true })).toBeVisible()
  page.once('dialog', dialog => dialog.accept())
  await page.getByRole('button', { name: 'Cancelar', exact: true }).first().click()
  await expect(page.getByText('Cancelado', { exact: true })).toBeVisible()
  await page.locator('#schedules > .section-heading').getByLabel('Estado').selectOption('cancelled')
  await expect(page.locator('.schedule-card').getByText('ui-lifecycle-edited', { exact: true })).toBeVisible()
  await page.locator('#schedules > .section-heading').getByLabel('Estado').selectOption('active')
  await expect(page.locator('.schedule-card').getByText('ui-lifecycle-edited', { exact: true })).toHaveCount(0)
})

// Covers: BDD-JRN-001-08 BDD-JRN-001-10 BDD-JRN-005-04
test('applies a saved recovery policy and displays consolidated missed times', async ({ page }) => {
  await page.goto(baseUrl)
  await createSchedule(page, 'ui-recovery')
  await page.getByRole('button', { name: 'Editar' }).click()
  const edit = page.locator('form.inline-form').first()
  await edit.getByLabel('Política de vencidas').selectOption('consolidate')
  await edit.getByRole('button', { name: 'Salvar alteração' }).click()
  await expect(page.getByRole('status')).toContainText('Alterações salvas.')
  const response = await fetch(`${baseUrl}/api/v1/schedules?limit=100`)
  const schedules = await response.json() as { items: Array<{ id: string; name: string }> }
  const id = schedules.items.find(item => item.name === 'ui-recovery')?.id
  expect(id).toBeTruthy()
  const currentMinute = Math.floor(Date.now() / 60_000) * 60_000_000
  const cursor = currentMinute - 5 * 60_000_000
  const python = process.platform === 'win32' ? 'python' : 'python3'
  execFileSync(python, ['-c', `import sqlite3,sys; db,id,cursor=sys.argv[1:]; c=sqlite3.connect(db); c.execute("UPDATE schedules SET timing_json=?,last_evaluated_at=? WHERE id=?", ('{"type":"cron","expression":"* * * * *"}',int(cursor),id)); c.commit()`, path.join(tempDir, 'web.sqlite3'), id!, String(cursor)])
  await page.getByRole('button', { name: 'Atualizar ↻' }).click()
  await expect(page.locator('.missed')).toContainText(/\d+ horários vencidos/, { timeout: 10_000 })
})

// Covers: BDD-JRN-001-12 BDD-JRN-005-06
 test('exposes accessible labels and keyboard focus on the creation flow', async ({ page }) => {
  await page.goto(baseUrl)
  await page.getByRole('button', { name: '＋ Novo schedule' }).focus()
  await page.keyboard.press('Enter')
  await expect(page.getByLabel(/Executável/)).toBeVisible()
  await expect(page.getByLabel(/Argumentos/)).toBeVisible()
  await expect(page.getByLabel(/Horários vencidos/)).toBeVisible()
  await page.getByLabel(/Executável/).focus()
  await expect(page.getByLabel(/Executável/)).toBeFocused()
})

// Covers: BDD-JRN-004-03
 test('shows a recoverable error when the schedules API fails', async ({ page }) => {
  await page.goto(baseUrl)
  await page.route('**/api/v1/schedules?limit=100', route => route.fulfill({ status: 503, body: 'unavailable' }))
  await page.getByRole('button', { name: /Atualizar/ }).click()
  await expect(page.getByRole('alert')).toBeVisible()
})

// Covers: BDD-JRN-004-01 BDD-JRN-004-02 BDD-JRN-016-05
test('shows an empty history, then the completed occurrence and its output', async ({ page }) => {
  await page.goto(baseUrl)
  await createSchedule(page, 'ui-history')
  await expect(page.getByText('Ainda não há execuções registradas.')).toBeVisible()

  await page.getByRole('button', { name: 'Executar agora' }).click()
  const row = page.locator('.occurrence-row').first()
  await expect(row).toContainText('Concluído', { timeout: 10_000 })
  await row.getByText('Ver saída').click()
  await expect(row.locator('pre').first()).toContainText('ui-ok')
  const response = await fetch(`${baseUrl}/api/v1/schedules?limit=100`)
  const schedules = await response.json() as { items: Array<{ id: string; name: string }> }
  const scheduleId = schedules.items.find(item => item.name === 'ui-history')?.id
  expect(scheduleId).toBeTruthy()
  const python = process.platform === 'win32' ? 'python' : 'python3'
  const insertRuns = 'import sqlite3,sys,time,uuid; db,sid=sys.argv[1:]; now=int(time.time()*1000000); c=sqlite3.connect(db); c.executemany("INSERT INTO occurrences(id,schedule_id,status,source,created_at,finished_at) VALUES(?,? ,\'succeeded\',\'manual\',?,?)", [(str(uuid.uuid4()),sid,now+i,now+i) for i in range(55)]); c.commit()'
  execFileSync(python, ['-c', insertRuns, path.join(tempDir, 'web.sqlite3'), scheduleId!])
  const scheduleDetails = page.locator('.schedule-details')
  await scheduleDetails.getByLabel('Estado').selectOption('succeeded')
  await expect(scheduleDetails.locator('.occurrence-row')).toHaveCount(50)
  await scheduleDetails.getByRole('button',{name:'Carregar mais deste schedule'}).click()
  await expect(scheduleDetails.locator('.occurrence-row')).toHaveCount(56)
  const historyParams = new URLSearchParams({limit:'50',status:'succeeded',scheduleId:scheduleId!,from:new Date(Date.now()-7*86400000).toISOString(),to:new Date().toISOString()})
  const firstPage = await (await fetch(`${baseUrl}/api/v1/occurrences?${historyParams}`)).json() as {items: unknown[];nextCursor?: string}
  expect(firstPage.items).toHaveLength(50); expect(firstPage.nextCursor).toBeTruthy()
  const secondPage = await (await fetch(`${baseUrl}/api/v1/occurrences?${historyParams}&cursor=${firstPage.nextCursor}`)).json() as {items: unknown[]}
  expect(secondPage.items.length).toBeGreaterThan(0)
  const history = page.locator('.history-panel')
  await history.getByLabel('Período').selectOption('7')
  await history.getByLabel('Estado').selectOption('succeeded')
  await history.getByLabel('Schedule ID').fill(scheduleId!)
  await history.getByRole('button', { name: 'Aplicar filtros' }).click()
  await expect(history.locator('.global-occurrence')).toHaveCount(50)
  const nextPageResponse = page.waitForResponse(response => response.url().includes('/api/v1/occurrences?') && new URL(response.url()).searchParams.has('cursor'))
  await history.getByRole('button', { name: 'Carregar mais execuções' }).click()
  expect(((await (await nextPageResponse).json()) as { items: unknown[] }).items.length).toBeGreaterThan(0)
  await expect(history.locator('.global-occurrence')).toHaveCount(56)
})

// Covers: BDD-JRN-008-01 BDD-JRN-008-02
 test('starts and cancels an active execution through the UI', async ({ page }) => {
  await page.goto(baseUrl)
  await createSchedule(page, 'ui-cancel', process.execPath, '["-e","setTimeout(() => {}, 30000)"]')
  await page.getByRole('button', { name: 'Executar agora' }).click()
  const row = page.locator('.occurrence-row').first()
  await expect(row).toContainText('Executando', { timeout: 10_000 })
  page.once('dialog', dialog => dialog.accept())
  await row.getByRole('button', { name: 'Cancelar' }).click()
  await expect(row).toContainText('Cancelado', { timeout: 10_000 })
})

// Covers: BDD-JRN-009-01
 test('retries a failed occurrence and displays the retry result', async ({ page }) => {
  await page.goto(baseUrl)
  await createSchedule(page, 'ui-retry', process.execPath, '["-e","process.exit(7)"]', 0)
  await page.getByRole('button', { name: 'Executar agora' }).click()
  const rows = page.locator('.occurrence-row')
  await expect(rows.first()).toContainText('Falhou', { timeout: 10_000 })
  await rows.first().getByRole('button', { name: 'Tentar novamente' }).click()
  await expect(rows).toHaveCount(2, { timeout: 10_000 })
  await expect(rows.first()).toContainText('Falhou', { timeout: 10_000 })
})
