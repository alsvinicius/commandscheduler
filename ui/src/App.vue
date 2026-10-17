<script setup lang="ts">
import { defineAsyncComponent, onMounted, onUnmounted, ref } from 'vue'

type Timing = { type: 'cron'; expression: string } | { type: 'once'; at: string }
type Schedule = {
  id: string; name?: string; program: string; args: string[]; timing: Timing; parameters: unknown
  missedPolicy: 'skip' | 'consolidate' | 'sequence'; status: 'active' | 'paused' | 'cancelled'
  parameterUpdateTrigger: { enabled: boolean; scheduleBehavior: 'replace' | 'coexist' }
  retryPolicy: { maxRetries: number; delaysSeconds: number[] }; createdAt: string; updatedAt: string
}
type Occurrence = {
  id: string; scheduleId: string; status: string; source: string; attempt: number
  createdAt: string; scheduledFor?: string | null; startedAt?: string | null; finishedAt?: string | null
  exitCode?: number | null; stdout?: string; stderr?: string; stdoutTruncated: boolean; stderrTruncated: boolean
  missedOccurrences?: { count: number; from: string; through: string }; retryOf?: string | null
}
type Page<T> = { items: T[]; nextCursor?: string | null }
type CommandTestResult = { exitCode: number | null; stdout: string; stderr: string; stdoutTruncated: boolean; stderrTruncated: boolean; timedOut: boolean }
type Overview = { schedules: { active: number; paused: number; cancelled: number }; completed: number; failed: number; next: null | { scheduleId: string; name?: string; program: string; at: string } }
type AgendaItem = { scheduleId: string; name?: string; program: string; at: string; occurrenceId?: string | null; status?: string | null }

const ApiReference = defineAsyncComponent(async () => {
  const [{ ApiReference: component }] = await Promise.all([
    import('@scalar/api-reference'),
    import('@scalar/api-reference/style.css'),
  ])
  return component
})
const isApiReference = window.location.pathname.startsWith('/docs/api')
const apiConfiguration = { url: '/openapi.yaml', theme: 'default' as const, layout: 'modern' as const, hideDarkModeToggle: true }

const schedules = ref<Schedule[]>([])
const scheduleCursor = ref<string | null>(null)
const occurrences = ref<Occurrence[]>([])
const occurrenceCursor = ref<string | null>(null)
const occurrenceFilter = ref('all')
const overview = ref<Overview | null>(null)
const agenda = ref<AgendaItem[]>([])
const globalRuns = ref<Occurrence[]>([])
const globalOutputs = ref<Record<string, Occurrence>>({})
const historyCursor = ref<string | null>(null)
const periodDays = ref(1)
const scheduleFilter = ref('all')
const runFilter = ref('all')
const historyDays = ref(30)
const historyScheduleId = ref('')
const agendaPeriod = ref<'day' | 'week' | 'month'>('week')
const selectedId = ref<string | null>(null)
const loading = ref(false)
const daemonHealthy = ref(false)
const saving = ref(false)
const testingCommand = ref(false)
const commandTestResult = ref<CommandTestResult | null>(null)
const error = ref('')
const notice = ref('')
const showCreate = ref(false)
const showEdit = ref(false)
const parameterDraft = ref('null')
const newSchedule = ref({ name: '', program: '', args: '[]', timingType: 'cron' as 'cron' | 'once', expression: '0 9 * * 1-5', at: '', parameters: 'null', missedPolicy: 'skip', triggerEnabled: false, triggerBehavior: 'coexist', maxRetries: 3, retryDelays: '60, 300, 900' })
const editDraft = ref({ name: '', program: '', args: '[]', timingType: 'cron' as 'cron' | 'once', expression: '', at: '', missedPolicy: 'skip', preset: 'advanced', interval: 5, hour: '09:00', dayOfMonth: 1, weekdays: [1,2,3,4,5] as number[] })
const recurrence = ref({ preset: 'weekly', interval: 5, hour: '09:00', dayOfMonth: 1, weekdays: [1,2,3,4,5] as number[] })
let refreshTimer: number | undefined
let historyRequestId = 0
let occurrenceRequestId = 0

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const response = await fetch(`/api/v1${path}`, { ...init, headers: { ...(init.body ? { 'Content-Type': 'application/json' } : {}), ...init.headers } })
  const contentType = response.headers.get('content-type') ?? ''
  const payload = contentType.includes('json') ? await response.json() : await response.text()
  if (!response.ok) throw new Error(typeof payload === 'object' && payload?.detail ? payload.detail : `Request failed (${response.status})`)
  return payload as T
}

async function refresh(reset = true) {
  try {
    const health = await fetch('/health'); daemonHealthy.value = health.ok && (await health.text()) === 'ok'
    if (!daemonHealthy.value) throw new Error('Daemon local indisponível.')
    if (reset) { schedules.value = []; scheduleCursor.value = null }
    const params = new URLSearchParams({limit:'100'}); if (scheduleCursor.value) params.set('cursor',scheduleCursor.value); if (scheduleFilter.value !== 'all') params.set('status',scheduleFilter.value)
    const page = await request<Page<Schedule>>(`/schedules?${params}`)
    schedules.value.push(...page.items); scheduleCursor.value = page.nextCursor ?? null
    if (reset && selectedId.value && !page.items.some(item => item.id === selectedId.value)) selectedId.value = null
    if (selectedId.value) await loadOccurrences(selectedId.value)
    await Promise.all([loadOverview(), loadAgenda()])
  } catch (cause) {
    if (!daemonHealthy.value) { overview.value = null; agenda.value = []; if (reset) schedules.value = [] }
    error.value = cause instanceof Error ? cause.message : 'Não foi possível consultar o daemon.'
  }
}
async function loadMoreSchedules() { await refresh(false) }

function intervalBounds(days: number) { const end = new Date(); const start = new Date(end); if (days === 1) start.setHours(0,0,0,0); else { start.setDate(start.getDate()-days+1); start.setHours(0,0,0,0) } return { from: start.toISOString(), to: end.toISOString() } }
async function loadOverview() { const range = intervalBounds(periodDays.value); overview.value = await request<Overview>(`/overview?from=${encodeURIComponent(range.from)}&to=${encodeURIComponent(range.to)}`) }
function agendaBounds() { const start = new Date(); start.setHours(0,0,0,0); const end = new Date(start); if (agendaPeriod.value === 'day') end.setDate(end.getDate()+1); else if (agendaPeriod.value === 'week') { start.setDate(start.getDate()-((start.getDay()+6)%7)); end.setTime(start.getTime()); end.setDate(end.getDate()+7) } else { start.setDate(1); end.setMonth(end.getMonth()+1); end.setDate(1) } return { from: start.toISOString(), to: end.toISOString() } }
async function loadAgenda() { const range = agendaBounds(); agenda.value = (await request<{items: AgendaItem[]}>(`/agenda?from=${encodeURIComponent(range.from)}&to=${encodeURIComponent(range.to)}`)).items }
async function loadGlobalRuns(reset = false) { if (reset) { historyRequestId++; globalRuns.value=[]; historyCursor.value=null }; const requestId=historyRequestId; const params = new URLSearchParams({limit:'50'}); if (historyCursor.value) params.set('cursor',historyCursor.value); if (runFilter.value !== 'all') params.set('status',runFilter.value); if(historyScheduleId.value.trim()) params.set('scheduleId',historyScheduleId.value.trim()); const historyFrom=new Date(); historyFrom.setDate(historyFrom.getDate()-historyDays.value+1); historyFrom.setHours(0,0,0,0); params.set('from',historyFrom.toISOString()); params.set('to',new Date().toISOString()); try { const page=await request<Page<Occurrence>>(`/occurrences?${params}`); if(requestId!==historyRequestId)return; globalRuns.value.push(...page.items); historyCursor.value=page.nextCursor ?? null } catch(cause) { if(requestId===historyRequestId)error.value=cause instanceof Error?cause.message:'Falha ao consultar o histórico.' } }
async function loadGlobalOutput(item:Occurrence) { if (globalOutputs.value[item.id]) { delete globalOutputs.value[item.id]; return }; try { globalOutputs.value[item.id]=await request<Occurrence>(`/occurrences/${item.id}`) } catch(cause) { error.value=cause instanceof Error?cause.message:'Não foi possível carregar a saída.' } }
async function updateDashboard() { try { await Promise.all([loadOverview(),loadAgenda()]) } catch (cause) { error.value=cause instanceof Error?cause.message:'Não foi possível atualizar o cockpit.' } }
async function openAgendaSchedule(id:string) { try { scheduleFilter.value='all'; await refresh(); let schedule=schedules.value.find(item=>item.id===id); if(!schedule) { schedule=await request<Schedule>(`/schedules/${id}`); schedules.value.unshift(schedule) }; selectedId.value=id; prepareScheduleDraft(schedule); await loadOccurrences(id); document.querySelector('#schedules')?.scrollIntoView({behavior:'smooth'}) } catch(cause) { error.value=cause instanceof Error?cause.message:'Não foi possível abrir o schedule.' } }
async function refreshLive() { try { const health=await fetch('/health'); daemonHealthy.value=health.ok && (await health.text())==='ok'; if(!daemonHealthy.value) throw new Error('Daemon local indisponível.'); await Promise.all([loadOverview(),loadAgenda(),...(selectedId.value?[refreshSelectedOccurrences(selectedId.value)]:[])]) } catch(cause) { daemonHealthy.value=false; overview.value=null; agenda.value=[]; error.value=cause instanceof Error?cause.message:'Daemon local indisponível.' } }

async function loadOccurrences(scheduleId: string, reset = true) {
  const requestId = ++occurrenceRequestId
  const filter = occurrenceFilter.value
  if(reset){ occurrences.value=[]; occurrenceCursor.value=null }
  try { const params=new URLSearchParams({limit:'50'}); if(occurrenceCursor.value)params.set('cursor',occurrenceCursor.value); if(filter!=='all')params.set('status',filter); const page=await request<Page<Occurrence>>(`/schedules/${scheduleId}/occurrences?${params}`); if(requestId!==occurrenceRequestId || selectedId.value!==scheduleId || occurrenceFilter.value!==filter)return false; if(reset)occurrences.value=page.items; else { const known=new Set(occurrences.value.map(item=>item.id)); occurrences.value.push(...page.items.filter(item=>!known.has(item.id))) }; occurrenceCursor.value=page.nextCursor??null; return true }
  catch (cause) { if(requestId===occurrenceRequestId && selectedId.value===scheduleId && occurrenceFilter.value===filter)error.value = cause instanceof Error ? cause.message : 'Falha ao consultar execuções.'; return false }
}
async function reloadOccurrences() { if(selectedId.value) await loadOccurrences(selectedId.value,true) }
async function refreshSelectedOccurrences(id:string) { const filter=occurrenceFilter.value; const existing=[...occurrences.value]; const applied=await loadOccurrences(id,true); if(!applied || selectedId.value!==id || occurrenceFilter.value!==filter)return; const known=new Set(occurrences.value.map(item=>item.id)); occurrences.value.push(...existing.filter(item=>item.scheduleId===id && (filter==='all'||item.status===filter) && !known.has(item.id))) }

function chooseSchedule(schedule: Schedule) {
  occurrenceRequestId++
  selectedId.value = selectedId.value === schedule.id ? null : schedule.id
  error.value = ''
  if (selectedId.value) {
    prepareScheduleDraft(schedule)
    void loadOccurrences(schedule.id)
  } else occurrences.value = []
}

function prepareScheduleDraft(schedule: Schedule) {
  parameterDraft.value = JSON.stringify(schedule.parameters, null, 2)
  const decoded = schedule.timing.type === 'cron' ? decodePreset(schedule.timing.expression) : null
  editDraft.value = {
    name: schedule.name ?? '', program: schedule.program, args: JSON.stringify(schedule.args, null, 2),
    timingType: schedule.timing.type, expression: schedule.timing.type === 'cron' ? schedule.timing.expression : '',
    at: schedule.timing.type === 'once' ? toLocalInput(schedule.timing.at) : '', missedPolicy: schedule.missedPolicy,
    preset: decoded?.preset ?? 'advanced', interval: decoded?.interval ?? 5, hour: decoded?.hour ?? '09:00', dayOfMonth: decoded?.dayOfMonth ?? 1, weekdays: decoded?.weekdays ?? [1,2,3,4,5],
  }
}

async function createSchedule() {
  error.value = ''; saving.value = true
  try {
    if (looksLikeWholeCommand(newSchedule.value.program)) throw new Error('No campo Executável, informe apenas o programa. Mova as opções e o código para Argumentos.')
    if (newSchedule.value.timingType === 'cron' && recurrence.value.preset === 'weekly' && recurrence.value.weekdays.length === 0) throw new Error('Marque ao menos um dia da semana.')
    const args: unknown = JSON.parse(newSchedule.value.args)
    const parameters: unknown = JSON.parse(newSchedule.value.parameters)
    if (!Array.isArray(args) || args.some(item => typeof item !== 'string')) throw new Error('Os argumentos precisam ser um array JSON de strings.')
    const delays = newSchedule.value.retryDelays.split(',').map(value => Number(value.trim())).filter(Number.isFinite)
    const timing: Timing = newSchedule.value.timingType === 'cron'
      ? { type: 'cron', expression: recurrence.value.preset === 'advanced' ? newSchedule.value.expression.trim() : encodePreset(recurrence.value) }
      : { type: 'once', at: new Date(newSchedule.value.at).toISOString() }
    const payload = {
      ...(newSchedule.value.name.trim() ? { name: newSchedule.value.name.trim() } : {}),
      program: newSchedule.value.program, args, timing, parameters,
      missedPolicy: newSchedule.value.missedPolicy,
      parameterUpdateTrigger: { enabled: newSchedule.value.triggerEnabled, scheduleBehavior: newSchedule.value.triggerBehavior },
      retryPolicy: { maxRetries: Number(newSchedule.value.maxRetries), delaysSeconds: delays },
    }
    const schedule = await request<Schedule>('/schedules', { method: 'POST', body: JSON.stringify(payload) })
    showCreate.value = false; showEdit.value = false; notice.value = 'Schedule criado.'; selectedId.value = schedule.id
    prepareScheduleDraft(schedule)
    await refresh()
  } catch (cause) { error.value = cause instanceof Error ? cause.message : 'Não foi possível criar o schedule.' }
  finally { saving.value = false }
}

async function testNewCommand() {
  error.value = ''; commandTestResult.value = null; testingCommand.value = true
  try {
    if (looksLikeWholeCommand(newSchedule.value.program)) throw new Error('No campo Executável, informe apenas o programa. Mova as opções e o código para Argumentos.')
    const args: unknown = JSON.parse(newSchedule.value.args)
    const parameters: unknown = JSON.parse(newSchedule.value.parameters)
    if (!Array.isArray(args) || args.some(item => typeof item !== 'string')) throw new Error('Os argumentos precisam ser um array JSON de strings.')
    commandTestResult.value = await request<CommandTestResult>('/commands/test', {
      method: 'POST', body: JSON.stringify({ program: newSchedule.value.program, args, parameters }),
    })
  } catch (cause) { error.value = cause instanceof Error ? cause.message : 'Não foi possível testar o comando.' }
  finally { testingCommand.value = false }
}

async function updateSchedule() {
  if (!selectedId.value) return
  error.value = ''; saving.value = true
  try {
    if (looksLikeWholeCommand(editDraft.value.program)) throw new Error('No campo Executável, informe apenas o programa. Mova as opções e o código para Argumentos.')
    if (editDraft.value.timingType === 'cron' && editDraft.value.preset === 'weekly' && editDraft.value.weekdays.length === 0) throw new Error('Marque ao menos um dia da semana.')
    const args: unknown = JSON.parse(editDraft.value.args)
    if (!Array.isArray(args) || args.some(item => typeof item !== 'string')) throw new Error('Os argumentos precisam ser um array JSON de strings.')
    const timing: Timing = editDraft.value.timingType === 'cron'
      ? { type: 'cron', expression: editDraft.value.preset === 'advanced' ? editDraft.value.expression.trim() : encodePreset(editDraft.value) }
      : { type: 'once', at: new Date(editDraft.value.at).toISOString() }
    await request<Schedule>(`/schedules/${selectedId.value}`, { method: 'PATCH', body: JSON.stringify({ name: editDraft.value.name || undefined, program: editDraft.value.program, args, timing, missedPolicy: editDraft.value.missedPolicy }) })
    showEdit.value = false; notice.value = 'Alterações salvas.'; await refresh()
  } catch (cause) { error.value = cause instanceof Error ? cause.message : 'Não foi possível atualizar o schedule.' }
  finally { saving.value = false }
}

function looksLikeWholeCommand(value: string) {
  const command = value.trim()
  if (!command) return false
  const optionAfterProgram = /^\S+\s+-{1,2}\S+/.test(command)
  const interpreterWithArgument = /^(?:(?:.*[/\\])?)(?:python(?:\d+(?:\.\d+)*)?|node|ruby|perl|php|bash|sh|zsh|cmd(?:\.exe)?|powershell(?:\.exe)?)\s+/i.test(command)
  return optionAfterProgram || interpreterWithArgument
}

async function saveParameters() {
  if (!selectedId.value) return
  error.value = ''; saving.value = true
  try {
    const parameters: unknown = JSON.parse(parameterDraft.value)
    const result = await request<{ occurrenceId?: string }>(`/schedules/${selectedId.value}/parameters`, { method: 'PUT', body: JSON.stringify(parameters) })
    notice.value = result.occurrenceId ? `Parâmetros atualizados; execução ${result.occurrenceId} iniciada.` : 'Parâmetros atualizados.'
    await refresh()
  } catch (cause) { error.value = cause instanceof Error ? cause.message : 'Parâmetros JSON inválidos.' }
  finally { saving.value = false }
}

async function scheduleAction(schedule: Schedule, action: 'pause' | 'resume' | 'cancel' | 'runs') {
  if (action === 'cancel' && !window.confirm('Cancelar este schedule? Execuções já iniciadas continuam.')) return
  try {
    const result = await request<Occurrence | Schedule>(`/schedules/${schedule.id}/${action}`, { method: 'POST' })
    notice.value = action === 'runs' ? `Execução ${((result as Occurrence).id)} enfileirada.` : `Schedule ${action === 'pause' ? 'pausado' : action === 'resume' ? 'retomado' : 'cancelado'}.`
    await refresh()
  } catch (cause) { error.value = cause instanceof Error ? cause.message : 'A ação não foi concluída.' }
}

async function occurrenceAction(item: Occurrence, action: 'cancel' | 'retry') {
  if (action === 'cancel' && !window.confirm('Solicitar o encerramento desta execução?')) return
  const confirmInterrupted = item.status === 'interrupted' && action === 'retry'
    ? window.confirm('A execução anterior foi interrompida. Confirma uma nova tentativa?') : false
  if (item.status === 'interrupted' && action === 'retry' && !confirmInterrupted) return
  try {
    await request<Occurrence>(`/occurrences/${item.id}/${action}`, { method: 'POST', ...(action === 'retry' ? { body: JSON.stringify({ confirmInterrupted }) } : {}) })
    notice.value = action === 'cancel' ? 'Pedido de cancelamento aceito.' : 'Nova tentativa enfileirada.'
    if (selectedId.value) await loadOccurrences(selectedId.value)
  } catch (cause) { error.value = cause instanceof Error ? cause.message : 'A ação não foi concluída.' }
}

function toLocalInput(value: string) {
  const date = new Date(value); const pad = (part: number) => `${part}`.padStart(2, '0')
  return `${date.getFullYear()}-${pad(date.getMonth()+1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`
}
function encodePreset(value: {preset:string;interval:number;hour:string;dayOfMonth:number;weekdays:number[]}) {
  const [h,m]=value.hour.split(':').map(Number)
  if(value.preset==='minutes') return `*/${value.interval} * * * *`
  if(value.preset==='hours') return `0 */${value.interval} * * *`
  if(value.preset==='daily') return `${m} ${h} * * *`
  if(value.preset==='weekly') return `${m} ${h} * * ${[...value.weekdays].sort((a,b)=>a-b).join(',')}`
  if(value.preset==='monthly') return `${m} ${h} ${value.dayOfMonth} * *`
  return ''
}
function decodePreset(expression:string) {
  let m=expression.match(/^\*\/(\d+) \* \* \* \*$/); if(m)return {preset:'minutes',interval:Number(m[1]),hour:'09:00',dayOfMonth:1,weekdays:[1,2,3,4,5]}
  m=expression.match(/^0 \*\/(\d+) \* \* \*$/); if(m)return {preset:'hours',interval:Number(m[1]),hour:'09:00',dayOfMonth:1,weekdays:[1,2,3,4,5]}
  m=expression.match(/^(\d+) (\d+) \* \* \*$/); if(m)return {preset:'daily',interval:5,hour:`${m[2].padStart(2,'0')}:${m[1].padStart(2,'0')}`,dayOfMonth:1,weekdays:[1,2,3,4,5]}
  m=expression.match(/^(\d+) (\d+) \* \* ((?:[0-6](?:-[0-6])?)(?:,(?:[0-6](?:-[0-6])?))*)$/); if(m) { const days=m[3].split(',').flatMap(token=>{const [a,b]=token.split('-').map(Number); return b===undefined?[a]:Array.from({length:Math.abs(b-a)+1},(_,i)=>Math.min(a,b)+i)}); if(new Set(days).size===days.length)return {preset:'weekly',interval:5,hour:`${m[2].padStart(2,'0')}:${m[1].padStart(2,'0')}`,dayOfMonth:1,weekdays:days} }
  m=expression.match(/^(\d+) (\d+) (\d{1,2}) \* \*$/); if(m)return {preset:'monthly',interval:5,hour:`${m[2].padStart(2,'0')}:${m[1].padStart(2,'0')}`,dayOfMonth:Number(m[3]),weekdays:[1,2,3,4,5]}
  return null
}
function formatDate(value?: string | null) { return value ? new Date(value).toLocaleString() : '—' }
function timingLabel(timing: Timing) { return timing.type === 'cron' ? timing.expression : `Uma vez · ${formatDate(timing.at)}` }
function statusLabel(status: string) { return ({ active: 'Ativo', paused: 'Pausado', cancelled: 'Cancelado', queued: 'Na fila', running: 'Executando', succeeded: 'Concluído', failed: 'Falhou', interrupted: 'Interrompido' } as Record<string,string>)[status] ?? status }

onMounted(() => { loading.value = true; void Promise.all([refresh(),loadGlobalRuns(true)]).finally(() => { loading.value = false }); refreshTimer = window.setInterval(() => void refreshLive(), 3000) })
onUnmounted(() => { if (refreshTimer) window.clearInterval(refreshTimer) })
</script>

<template>
  <ApiReference v-if="isApiReference" :configuration="apiConfiguration" />
  <main v-else class="app-shell instrument-design" data-theme="workshop">
    <header class="topbar">
      <a class="brand" href="/" aria-label="Command Scheduler, início"><span class="brand-mark" aria-hidden="true">↗</span><span>command<span class="brand-light">scheduler</span></span></a>
      <nav aria-label="Navegação principal"><a href="#schedules">Schedules</a><a href="/docs/api">API reference</a><span class="daemon" :class="{'daemon-offline':!daemonHealthy}"><i aria-hidden="true"></i>{{ daemonHealthy ? 'Daemon local' : 'Daemon indisponível' }}</span></nav>
    </header>

    <section class="cockpit" aria-label="Resumo operacional">
      <div class="section-heading"><div><p class="eyebrow">OPERAÇÃO LOCAL</p><h1>Cockpit</h1></div><div class="cockpit-controls"><label>Indicadores<select v-model.number="periodDays" @change="updateDashboard"><option :value="1">Hoje</option><option :value="7">7 dias</option><option :value="30">30 dias</option><option :value="90">90 dias</option></select></label><button class="primary" type="button" @click="showCreate = !showCreate">{{ showCreate ? 'Fechar formulário' : '＋ Novo schedule' }}</button></div></div>
      <div class="metric-grid"><article class="next-metric"><span>PRÓXIMO AGENDAMENTO</span><strong>{{ overview?.next?.name || overview?.next?.program || 'Nenhum agendamento ativo' }}</strong><small>{{ overview?.next ? formatDate(overview.next.at) : '—' }}</small></article><article><span>ATIVOS</span><strong>{{ overview?.schedules.active ?? '—' }}</strong></article><article><span>PAUSADOS</span><strong>{{ overview?.schedules.paused ?? '—' }}</strong></article><article><span>CANCELADOS</span><strong>{{ overview?.schedules.cancelled ?? '—' }}</strong></article><article><span>CONCLUÍDOS · PERÍODO</span><strong>{{ overview?.completed ?? '—' }}</strong></article><article><span>FALHAS · PERÍODO</span><strong class="failure-number">{{ overview?.failed ?? '—' }}</strong></article></div>
    </section>

    <section class="agenda-panel" aria-labelledby="agenda-title"><div class="section-heading"><div><p class="eyebrow">AGENDA</p><h2 id="agenda-title">Agendamentos e execuções</h2></div><div class="inline-controls"><label>Visualização<select v-model="agendaPeriod" @change="updateDashboard"><option value="day">Dia</option><option value="week">Semana</option><option value="month">Mês</option></select></label></div></div><div v-if="agenda.length" class="agenda-list"><article v-for="item in agenda" :key="`${item.scheduleId}-${item.at}-${item.occurrenceId||'scheduled'}`" class="agenda-item"><div class="agenda-item-content"><div class="agenda-item-heading"><button class="agenda-title" type="button" @click="openAgendaSchedule(item.scheduleId)">{{ item.name || item.program }}</button><span class="agenda-status" :class="`agenda-status-${item.status || 'scheduled'}`">{{ item.status ? statusLabel(item.status) : 'Agendado' }}</span></div><time class="agenda-time" :datetime="item.at">{{ formatDate(item.at) }}</time><code class="agenda-command" :title="item.program">{{ item.program }}</code></div></article></div><p v-else class="history-empty">Nenhum schedule ativo ou execução concluída neste período.</p></section>

    <section v-if="showCreate" class="form-panel" aria-labelledby="create-title">
      <div class="section-heading"><div><p class="eyebrow">CONFIGURAÇÃO</p><h2 id="create-title">Criar schedule</h2></div><button class="icon-button" type="button" aria-label="Fechar" @click="showCreate = false">×</button></div>
      <form class="schedule-form" @submit.prevent="createSchedule">
        <label>Nome <span>opcional</span><input v-model="newSchedule.name" placeholder="Ex.: Sincronizar relatórios"></label>
        <div class="form-row"><label>Executável <span>somente o programa</span><input v-model="newSchedule.program" required placeholder="/usr/bin/python3"><small>Informe o caminho ou nome do executável, sem opções nem código.</small></label><label>Argumentos <span>array JSON de strings</span><textarea v-model="newSchedule.args" required rows="2" spellcheck="false"></textarea><small>Exemplo Python: <code>["-c", "print('Hello, world!')"]</code></small></label></div>
        <p v-if="looksLikeWholeCommand(newSchedule.program)" class="command-input-warning" role="alert">Parece que você colou a linha de comando inteira em Executável. Use <code>/usr/bin/python3</code> aqui e coloque <code>-c</code> e o código em Argumentos.</p>
        <fieldset><legend>Quando executar</legend><label class="radio"><input v-model="newSchedule.timingType" type="radio" value="cron"> Recorrente</label><label class="radio"><input v-model="newSchedule.timingType" type="radio" value="once"> Uma vez</label>
          <template v-if="newSchedule.timingType === 'cron'"><label>Repetição<select v-model="recurrence.preset"><option value="minutes">A cada N minutos</option><option value="hours">A cada N horas</option><option value="daily">Diariamente</option><option value="weekly">Dias da semana</option><option value="monthly">Mensalmente</option><option value="advanced">Cron avançado</option></select></label><label v-if="['minutes','hours'].includes(recurrence.preset)">Intervalo<select v-model.number="recurrence.interval"><template v-if="recurrence.preset==='minutes'"><option v-for="n in [1,2,3,4,5,6,10,12,15,20,30]" :key="n" :value="n">A cada {{ n }} minutos</option></template><template v-else><option v-for="n in [1,2,3,4,6,8,12]" :key="n" :value="n">A cada {{ n }} horas</option></template></select></label><label v-if="['daily','weekly','monthly'].includes(recurrence.preset)">Horário local<input v-model="recurrence.hour" type="time" required></label><label v-if="recurrence.preset==='weekly'">Dias da semana <span>marque um ou mais</span><div class="weekday-picker"><label v-for="(day,index) in ['Dom','Seg','Ter','Qua','Qui','Sex','Sáb']" :key="day"><input v-model="recurrence.weekdays" type="checkbox" :value="index">{{ day }}</label></div></label><label v-if="recurrence.preset==='monthly'">Dia do mês<input v-model.number="recurrence.dayOfMonth" type="number" min="1" max="31" required></label><label v-if="recurrence.preset==='advanced'">Cron de cinco campos<input v-model="newSchedule.expression" required placeholder="30 8 * * 1-5"><small>Minuto hora dia-do-mês mês dia-da-semana · fuso local</small></label><small v-if="recurrence.preset!=='advanced'" class="cron-preview">Equivalente cron: <code>{{ encodePreset(recurrence) }}</code> · fuso local</small></template>
          <label v-else>Data e hora local<input v-model="newSchedule.at" required type="datetime-local"></label>
        </fieldset>
        <div class="form-row"><label>Horários vencidos<select v-model="newSchedule.missedPolicy"><option value="skip">Pular</option><option value="consolidate">Consolidar em uma execução</option><option value="sequence">Executar em sequência</option></select></label><label>Parâmetros <span>qualquer JSON, até 64 KiB</span><textarea v-model="newSchedule.parameters" rows="3" spellcheck="false"></textarea></label></div>
        <details><summary>Opções de execução</summary><div class="form-row"><label>Tentativas adicionais<input v-model.number="newSchedule.maxRetries" type="number" min="0"></label><label>Intervalos em segundos<input v-model="newSchedule.retryDelays" placeholder="60, 300, 900"></label></div><label class="check"><input v-model="newSchedule.triggerEnabled" type="checkbox"> Executar também quando os parâmetros forem atualizados</label><label v-if="newSchedule.triggerEnabled">Comportamento do gatilho<select v-model="newSchedule.triggerBehavior"><option value="coexist">Coexistir com horário</option><option value="replace">Substituir ocorrência agendada na fila</option></select></label></details>
        <aside class="command-test-notice" role="note">Testar o comando executa o programa agora, com os argumentos e parâmetros informados. Ele pode alterar arquivos ou produzir outros efeitos. O teste não cria schedule nem entra no histórico e é encerrado após 30 segundos.</aside>
        <div class="form-actions"><button class="secondary" type="button" :disabled="saving || testingCommand || !newSchedule.program.trim() || looksLikeWholeCommand(newSchedule.program)" @click="testNewCommand">{{ testingCommand ? 'Testando…' : '▶ Testar comando' }}</button><button class="primary" type="submit" :disabled="saving || testingCommand || looksLikeWholeCommand(newSchedule.program)">{{ saving ? 'Salvando…' : 'Criar schedule' }}</button></div>
        <section v-if="commandTestResult" class="command-test-result" aria-live="polite"><h3>Resultado do teste <span>{{ commandTestResult.timedOut ? '· tempo limite' : commandTestResult.exitCode === null ? '· não iniciou' : `· código ${commandTestResult.exitCode}` }}</span></h3><p v-if="commandTestResult.timedOut" class="test-timeout">O processo foi encerrado após 30 segundos.</p><div v-if="commandTestResult.stdout || commandTestResult.stdoutTruncated"><strong>stdout <small v-if="commandTestResult.stdoutTruncated">· truncado em 1 MiB</small></strong><pre>{{ commandTestResult.stdout }}</pre></div><div v-if="commandTestResult.stderr || commandTestResult.stderrTruncated"><strong>stderr <small v-if="commandTestResult.stderrTruncated">· truncado em 1 MiB</small></strong><pre class="stderr">{{ commandTestResult.stderr }}</pre></div><p v-if="!commandTestResult.stdout && !commandTestResult.stderr && !commandTestResult.timedOut">O comando não produziu saída.</p></section>
      </form>
    </section>

    <section id="schedules" class="workspace">
      <div class="section-heading"><div><p class="eyebrow">CONFIGURAÇÃO</p><h2>Schedules <span class="count">{{ schedules.length.toString().padStart(2, '0') }}</span></h2></div><div class="inline-controls"><label>Estado<select v-model="scheduleFilter" @change="refresh()"><option value="all">Todos</option><option value="active">Ativos</option><option value="paused">Inativos · pausados</option><option value="cancelled">Cancelados</option></select></label><button class="quiet-button" type="button" @click="refresh()">Atualizar ↻</button></div></div>
      <p v-if="notice" class="notice" role="status">{{ notice }} <button type="button" aria-label="Dispensar mensagem" @click="notice = ''">×</button></p>
      <p v-if="error" class="alert" role="alert">{{ error }}</p>
      <div v-if="loading && schedules.length === 0" class="empty">Conectando ao daemon local…</div>
      <div v-else-if="schedules.length === 0" class="empty"><span class="empty-mark" aria-hidden="true">⌁</span><h3>Nenhum schedule ainda</h3><p>Crie o primeiro e deixe uma tarefa recorrente sair da sua lista mental.</p><button class="text-button" type="button" @click="showCreate = true">Criar schedule <span aria-hidden="true">→</span></button></div>
      <div v-else class="schedule-list">
        <article v-for="schedule in schedules" :key="schedule.id" class="schedule-card" :class="{ selected: selectedId === schedule.id }">
          <button class="schedule-summary" type="button" :aria-expanded="selectedId === schedule.id" @click="chooseSchedule(schedule)">
            <span class="status-dot" :class="`status-${schedule.status}`" aria-hidden="true"></span>
            <span class="summary-main"><strong>{{ schedule.name || schedule.program.split('/').pop() }}</strong><span>{{ schedule.program }} <template v-if="schedule.args.length">{{ schedule.args.join(' ') }}</template></span></span>
            <code>{{ timingLabel(schedule.timing) }}</code><span class="status-label">{{ statusLabel(schedule.status) }}</span><span class="chevron" aria-hidden="true">{{ selectedId === schedule.id ? '−' : '+' }}</span>
          </button>
          <div v-if="selectedId === schedule.id" class="schedule-details">
            <div class="detail-actions"><button type="button" @click="scheduleAction(schedule, 'runs')">▶ Executar agora</button><button v-if="schedule.status === 'active'" type="button" @click="scheduleAction(schedule, 'pause')">Ⅱ Pausar</button><button v-if="schedule.status === 'paused'" type="button" @click="scheduleAction(schedule, 'resume')">▶ Retomar</button><button type="button" @click="showEdit = !showEdit">Editar</button><button class="danger-link" type="button" :disabled="schedule.status === 'cancelled'" @click="scheduleAction(schedule, 'cancel')">Cancelar</button></div>
            <form v-if="showEdit" class="inline-form" @submit.prevent="updateSchedule"><h3>Editar configuração</h3><div class="form-row"><label>Nome<input v-model="editDraft.name"></label><label>Executável <span>somente o programa</span><input v-model="editDraft.program" required><small>Sem opções ou código; use Argumentos abaixo.</small></label></div><p v-if="looksLikeWholeCommand(editDraft.program)" class="command-input-warning" role="alert">Parece que você colou a linha de comando inteira em Executável. Separe o programa dos argumentos.</p><label>Argumentos <span>array JSON de strings</span><textarea v-model="editDraft.args" rows="2" spellcheck="false"></textarea><small>Exemplo Python: <code>["-c", "print('Hello, world!')"]</code></small></label><fieldset><legend>Horário</legend><label class="radio"><input v-model="editDraft.timingType" type="radio" value="cron"> Recorrente</label><label class="radio"><input v-model="editDraft.timingType" type="radio" value="once"> Uma vez</label><template v-if="editDraft.timingType==='cron'"><label>Repetição<select v-model="editDraft.preset"><option value="minutes">A cada N minutos</option><option value="hours">A cada N horas</option><option value="daily">Diariamente</option><option value="weekly">Dias da semana</option><option value="monthly">Mensalmente</option><option value="advanced">Cron avançado</option></select></label><label v-if="['minutes','hours'].includes(editDraft.preset)">Intervalo<select v-model.number="editDraft.interval"><option v-for="n in (editDraft.preset==='minutes'?[1,2,3,4,5,6,10,12,15,20,30]:[1,2,3,4,6,8,12])" :key="n" :value="n">A cada {{ n }} {{ editDraft.preset==='minutes'?'minutos':'horas' }}</option></select></label><label v-if="['daily','weekly','monthly'].includes(editDraft.preset)">Horário local<input v-model="editDraft.hour" type="time"></label><label v-if="editDraft.preset==='weekly'">Dias da semana<div class="weekday-picker"><label v-for="(day,index) in ['Dom','Seg','Ter','Qua','Qui','Sex','Sáb']" :key="day"><input v-model="editDraft.weekdays" type="checkbox" :value="index">{{ day }}</label></div></label><label v-if="editDraft.preset==='monthly'">Dia do mês<input v-model.number="editDraft.dayOfMonth" type="number" min="1" max="31"></label><label v-if="editDraft.preset==='advanced'">Cron<input v-model="editDraft.expression" required></label><small v-if="editDraft.preset!=='advanced'" class="cron-preview">Equivalente cron: <code>{{ encodePreset(editDraft) }}</code> · fuso local</small></template><label v-else>Data e hora local<input v-model="editDraft.at" required type="datetime-local"></label></fieldset><label>Política de vencidas<select v-model="editDraft.missedPolicy"><option value="skip">Pular</option><option value="consolidate">Consolidar</option><option value="sequence">Sequenciar</option></select></label><button class="primary" type="submit" :disabled="saving || looksLikeWholeCommand(editDraft.program)">Salvar alteração</button></form>
            <form class="inline-form params-form" @submit.prevent="saveParameters"><label>Parâmetros atuais<textarea v-model="parameterDraft" rows="4" spellcheck="false"></textarea></label><button class="secondary" type="submit" :disabled="saving">Atualizar parâmetros</button></form>
            <div class="history-heading"><h3>Execuções</h3><div class="inline-controls"><label>Estado<select v-model="occurrenceFilter" @change="reloadOccurrences"><option value="all">Todos</option><option value="queued">Na fila</option><option value="running">Executando</option><option value="succeeded">Concluídos</option><option value="failed">Falhas</option><option value="interrupted">Interrompidos</option><option value="cancelled">Cancelados</option></select></label><span>{{ occurrences.length }} carregadas</span></div></div>
            <div v-if="occurrences.length === 0" class="history-empty">Ainda não há execuções registradas.</div>
            <article v-for="item in occurrences" :key="item.id" class="occurrence-row">
              <span class="status-dot" :class="`status-${item.status}`" aria-hidden="true"></span><div class="occurrence-main"><strong>{{ statusLabel(item.status) }} <small>· {{ item.source }} · tentativa {{ item.attempt }}</small></strong><span>Agendada {{ formatDate(item.scheduledFor) }} <template v-if="item.exitCode !== null && item.exitCode !== undefined">· código {{ item.exitCode }}</template></span>
                <span v-if="item.missedOccurrences" class="missed">{{ item.missedOccurrences.count }} horários vencidos · {{ formatDate(item.missedOccurrences.from) }} — {{ formatDate(item.missedOccurrences.through) }}</span>
                <details v-if="item.stdout !== undefined || item.stderr !== undefined"><summary>Ver saída <span v-if="item.stdoutTruncated || item.stderrTruncated">· truncada em 1 MiB</span></summary><pre v-if="item.stdout">{{ item.stdout }}</pre><pre v-if="item.stderr" class="stderr">{{ item.stderr }}</pre></details>
              </div><div class="occurrence-actions"><button v-if="item.status === 'running' || item.status === 'queued'" type="button" @click="occurrenceAction(item, 'cancel')">Cancelar</button><button v-if="item.status === 'failed' || item.status === 'interrupted'" type="button" @click="occurrenceAction(item, 'retry')">Tentar novamente</button></div>
            </article>
            <button v-if="occurrenceCursor" class="secondary load-more" type="button" @click="loadOccurrences(schedule.id,false)">Carregar mais deste schedule</button>
          </div>
        </article>
      </div>
      <button v-if="scheduleCursor" class="secondary load-more" type="button" @click="loadMoreSchedules">Carregar mais schedules</button>
    </section>
    <section class="history-panel"><div class="section-heading"><div><p class="eyebrow">HISTÓRICO GLOBAL</p><h2>Execuções</h2></div><div class="inline-controls"><label>Período<select v-model.number="historyDays"><option :value="7">7 dias</option><option :value="30">30 dias</option><option :value="90">90 dias</option></select></label><label>Estado<select v-model="runFilter"><option value="all">Todos</option><option value="queued">Na fila</option><option value="running">Executando</option><option value="succeeded">Concluídos</option><option value="failed">Falhas</option><option value="interrupted">Interrompidos</option><option value="cancelled">Cancelados</option></select></label><label>Schedule ID<input v-model="historyScheduleId" placeholder="Filtrar por schedule"></label><button class="secondary" type="button" @click="loadGlobalRuns(true)">Aplicar filtros</button></div></div><p v-if="globalRuns.length===0" class="history-empty">Nenhuma execução encontrada para este filtro.</p><article v-for="item in globalRuns" :key="item.id" class="occurrence-row global-occurrence"><span class="status-dot" :class="`status-${item.status}`"></span><div class="occurrence-main"><strong>{{ statusLabel(item.status) }} <small>· {{ item.source }} · tentativa {{ item.attempt }} · schedule {{ item.scheduleId.slice(0,8) }}</small></strong><span>Agendada {{ formatDate(item.scheduledFor) }} <template v-if="item.finishedAt">· finalizada {{ formatDate(item.finishedAt) }}</template><template v-if="item.exitCode!==null && item.exitCode!==undefined">· código {{ item.exitCode }}</template></span><button class="output-link" type="button" @click="loadGlobalOutput(item)">{{ globalOutputs[item.id]?'Ocultar saída':'Ver saída' }}</button><template v-if="globalOutputs[item.id]"><pre v-if="globalOutputs[item.id].stdout">{{ globalOutputs[item.id].stdout }}</pre><pre v-if="globalOutputs[item.id].stderr" class="stderr">{{ globalOutputs[item.id].stderr }}</pre></template></div><div class="occurrence-actions"><button v-if="item.status==='running'||item.status==='queued'" type="button" @click="occurrenceAction(item,'cancel')">Cancelar</button><button v-if="item.status==='failed'||item.status==='interrupted'" type="button" @click="occurrenceAction(item,'retry')">Tentar novamente</button></div></article><button v-if="historyCursor" class="secondary load-more" type="button" @click="loadGlobalRuns()">Carregar mais execuções</button></section>
    <footer><span>COMMAND SCHEDULER <span class="footer-dot">·</span> LOCAL FIRST</span><a href="/docs/api">API reference</a><a href="/health" target="_blank" rel="noreferrer">Estado do daemon</a></footer>
  </main>
</template>
