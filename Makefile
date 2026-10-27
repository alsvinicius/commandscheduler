-include .project-gallery.mk

SHELL := /bin/sh

DEV_DIR := .tmp/dev
DEV_API_PORT ?= $(if $(COMMAND_SCHEDULER_API_PORT),$(COMMAND_SCHEDULER_API_PORT),30002)
DEV_WEB_PORT ?= $(if $(COMMAND_SCHEDULER_WEB_PORT),$(COMMAND_SCHEDULER_WEB_PORT),30014)
DEV_API_URL := http://127.0.0.1:$(DEV_API_PORT)
BACKEND_PID := $(DEV_DIR)/backend.pid
FRONTEND_PID := $(DEV_DIR)/frontend.pid
BACKEND_LOG := $(DEV_DIR)/backend.log
FRONTEND_LOG := $(DEV_DIR)/frontend.log
DEV_DB := $(DEV_DIR)/command-scheduler.sqlite

.PHONY: backend frontend start stop up down

start: up ## Inicia backend e frontend para a Project Gallery

stop: down ## Para backend e frontend para a Project Gallery

backend:
	@mkdir -p $(DEV_DIR)
	COMMAND_SCHEDULER_ADDR=127.0.0.1:$(DEV_API_PORT) COMMAND_SCHEDULER_DB_PATH=$(DEV_DB) cargo run --bin command-scheduler

frontend:
	COMMAND_SCHEDULER_API_URL=$(DEV_API_URL) pnpm --filter @command-scheduler/ui exec vite --host 127.0.0.1 --port $(DEV_WEB_PORT) --strictPort

up:
	@mkdir -p $(DEV_DIR)
	@if [ -f $(BACKEND_PID) ] && kill -0 "$$(cat $(BACKEND_PID))" 2>/dev/null; then echo "Backend já está rodando (PID $$(cat $(BACKEND_PID)))."; else \
		rm -f $(BACKEND_PID); \
		nohup setsid env COMMAND_SCHEDULER_ADDR=127.0.0.1:$(DEV_API_PORT) COMMAND_SCHEDULER_DB_PATH=$(DEV_DB) cargo run --bin command-scheduler >$(BACKEND_LOG) 2>&1 </dev/null & echo $$! >$(BACKEND_PID); \
		echo "Backend iniciado (PID $$(cat $(BACKEND_PID))); log: $(BACKEND_LOG)"; \
	fi
	@if [ -f $(FRONTEND_PID) ] && kill -0 "$$(cat $(FRONTEND_PID))" 2>/dev/null; then echo "Frontend já está rodando (PID $$(cat $(FRONTEND_PID)))."; else \
		rm -f $(FRONTEND_PID); \
		nohup setsid env COMMAND_SCHEDULER_API_URL=$(DEV_API_URL) pnpm --filter @command-scheduler/ui exec vite --host 127.0.0.1 --port $(DEV_WEB_PORT) --strictPort >$(FRONTEND_LOG) 2>&1 </dev/null & echo $$! >$(FRONTEND_PID); \
		echo "Frontend iniciado (PID $$(cat $(FRONTEND_PID))); log: $(FRONTEND_LOG)"; \
	fi
	@i=0; until curl --fail --silent $(DEV_API_URL)/health >/dev/null; do \
		i=$$((i + 1)); \
		if [ $$i -ge 60 ] || ! kill -0 "$$(cat $(BACKEND_PID))" 2>/dev/null; then echo "Backend não iniciou; consulte $(BACKEND_LOG)."; exit 1; fi; \
		sleep 1; \
	done
	@i=0; until curl --fail --silent http://127.0.0.1:$(DEV_WEB_PORT)/ >/dev/null; do \
		i=$$((i + 1)); \
		if [ $$i -ge 60 ] || ! kill -0 "$$(cat $(FRONTEND_PID))" 2>/dev/null; then echo "Frontend não iniciou; consulte $(FRONTEND_LOG)."; exit 1; fi; \
		sleep 1; \
	done
	@echo "Ambiente disponível: UI http://127.0.0.1:$(DEV_WEB_PORT) — API $(DEV_API_URL)"

down:
	@for service in frontend backend; do \
		pid_file=$(DEV_DIR)/$$service.pid; \
		if [ -f $$pid_file ]; then \
			pid=$$(cat $$pid_file); \
			if kill -0 $$pid 2>/dev/null; then kill -TERM -$$pid && echo "$$service encerrado (PID $$pid e processos do grupo)."; else echo "$$service já estava parado."; fi; \
			rm -f $$pid_file; \
		else echo "$$service não estava iniciado por make up."; fi; \
	done
