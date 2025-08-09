# Research Platform Docker Management Makefile

# Default target
.DEFAULT_GOAL := help

# Variables
COMPOSE_FILE := docker-compose.yml
COMPOSE_DEV_FILE := docker-compose.override.yml
PROJECT_NAME := research-platform

# Colors for output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[1;33m
BLUE := \033[0;34m
NC := \033[0m # No Color

.PHONY: help build up down logs clean dev prod monitoring postgres cache reset health status

## Display this help message
help:
	@echo "${BLUE}Research Platform Docker Management${NC}"
	@echo ""
	@echo "${YELLOW}Available commands:${NC}"
	@awk 'BEGIN {FS = ":.*##"} /^[a-zA-Z_-]+:.*##/ { printf "  ${GREEN}%-15s${NC} %s\n", $$1, $$2 }' $(MAKEFILE_LIST)
	@echo ""
	@echo "${YELLOW}Profiles:${NC}"
	@echo "  ${GREEN}dev${NC}          Development mode with live reloading"
	@echo "  ${GREEN}prod${NC}         Production mode with nginx reverse proxy"
	@echo "  ${GREEN}postgres${NC}     Use PostgreSQL instead of SQLite"
	@echo "  ${GREEN}cache${NC}        Enable Redis caching"
	@echo "  ${GREEN}monitoring${NC}   Enable Prometheus and Grafana monitoring"

## Build all Docker images
build:
	@echo "${BLUE}Building Docker images...${NC}"
	docker-compose -p $(PROJECT_NAME) build

## Build without cache
build-fresh: ## Build all images from scratch (no cache)
	@echo "${BLUE}Building Docker images from scratch...${NC}"
	docker-compose -p $(PROJECT_NAME) build --no-cache

## Start all services in detached mode
up:
	@echo "${BLUE}Starting services...${NC}"
	docker-compose -p $(PROJECT_NAME) up -d

## Start development environment
dev: ## Start development environment with live reloading
	@echo "${BLUE}Starting development environment...${NC}"
	docker-compose -p $(PROJECT_NAME) -f $(COMPOSE_FILE) -f $(COMPOSE_DEV_FILE) --profile dev up -d

## Start production environment
prod: ## Start production environment with nginx
	@echo "${BLUE}Starting production environment...${NC}"
	docker-compose -p $(PROJECT_NAME) --profile production up -d

## Start with PostgreSQL database
postgres: ## Start with PostgreSQL instead of SQLite
	@echo "${BLUE}Starting with PostgreSQL database...${NC}"
	docker-compose -p $(PROJECT_NAME) --profile postgres up -d

## Start with Redis caching
cache: ## Start with Redis caching enabled
	@echo "${BLUE}Starting with Redis caching...${NC}"
	docker-compose -p $(PROJECT_NAME) --profile cache up -d

## Start monitoring stack
monitoring: ## Start with monitoring (Prometheus + Grafana)
	@echo "${BLUE}Starting monitoring stack...${NC}"
	docker-compose -p $(PROJECT_NAME) --profile monitoring up -d

## Start full production stack
full: ## Start full production stack (postgres + cache + monitoring + nginx)
	@echo "${BLUE}Starting full production stack...${NC}"
	docker-compose -p $(PROJECT_NAME) --profile production --profile postgres --profile cache --profile monitoring up -d

## Stop all services
down:
	@echo "${BLUE}Stopping services...${NC}"
	docker-compose -p $(PROJECT_NAME) down

## View logs from all services
logs:
	docker-compose -p $(PROJECT_NAME) logs -f

## View logs from specific service
logs-backend: ## View web backend logs
	docker-compose -p $(PROJECT_NAME) logs -f web-backend

logs-app: ## View Xilem app logs  
	docker-compose -p $(PROJECT_NAME) logs -f xilem-app

logs-jupyter: ## View Jupyter logs
	docker-compose -p $(PROJECT_NAME) logs -f python-client

## Check service health status
health:
	@echo "${BLUE}Checking service health...${NC}"
	@docker-compose -p $(PROJECT_NAME) ps
	@echo ""
	@echo "${YELLOW}Health checks:${NC}"
	@docker inspect $(PROJECT_NAME)_web-backend_1 --format='{{.State.Health.Status}}' 2>/dev/null | sed 's/^/  web-backend: /' || echo "  web-backend: not running"
	@docker inspect $(PROJECT_NAME)_xilem-app_1 --format='{{.State.Health.Status}}' 2>/dev/null | sed 's/^/  xilem-app: /' || echo "  xilem-app: not running"
	@docker inspect $(PROJECT_NAME)_python-client_1 --format='{{.State.Health.Status}}' 2>/dev/null | sed 's/^/  python-client: /' || echo "  python-client: not running"

## Show service status
status:
	@echo "${BLUE}Service Status:${NC}"
	@docker-compose -p $(PROJECT_NAME) ps --format "table {{.Name}}\t{{.State}}\t{{.Ports}}"

## Clean up containers and images
clean:
	@echo "${YELLOW}Cleaning up containers and images...${NC}"
	docker-compose -p $(PROJECT_NAME) down -v
	docker system prune -f

## Reset everything (WARNING: destroys all data)
reset: ## Remove all containers, volumes, and images (destructive!)
	@echo "${RED}WARNING: This will destroy all data!${NC}"
	@read -p "Are you sure? [y/N] " -n 1 -r; echo; if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		docker-compose -p $(PROJECT_NAME) down -v; \
		docker volume prune -f; \
		docker image prune -af; \
		echo "${GREEN}Reset complete${NC}"; \
	else \
		echo "Cancelled"; \
	fi

## Run database migrations
migrate: ## Run database migrations
	@echo "${BLUE}Running database migrations...${NC}"
	docker-compose -p $(PROJECT_NAME) exec web-backend ./web-backend migrate

## Access backend container shell
shell-backend: ## Open shell in web backend container
	docker-compose -p $(PROJECT_NAME) exec web-backend /bin/bash

## Access Python client container shell  
shell-python: ## Open shell in Python client container
	docker-compose -p $(PROJECT_NAME) exec python-client /bin/bash

## Create development data directories
init-dev: ## Create development data directories
	@echo "${BLUE}Creating development directories...${NC}"
	mkdir -p dev-data/backend
	mkdir -p dev-data/notebooks
	mkdir -p ssl
	@echo "${GREEN}Development directories created${NC}"

## Display service URLs
urls: ## Display service URLs
	@echo "${BLUE}Service URLs:${NC}"
	@echo "  ${GREEN}Web API:${NC}       http://localhost:8080"
	@echo "  ${GREEN}Jupyter Lab:${NC}   http://localhost:8888"
	@echo "  ${GREEN}VNC (Xilem):${NC}   vnc://localhost:5900"
	@echo "  ${GREEN}Prometheus:${NC}    http://localhost:9090 (monitoring profile)"
	@echo "  ${GREEN}Grafana:${NC}       http://localhost:3000 (monitoring profile)"
	@echo "  ${GREEN}PostgreSQL:${NC}    localhost:5432 (postgres profile)"
	@echo "  ${GREEN}Redis:${NC}         localhost:6379 (cache profile)"

## Quick development setup
quick-dev: build init-dev dev urls ## Build and start development environment

## Test deployment
test: ## Run basic deployment test
	@echo "${BLUE}Testing deployment...${NC}"
	$(MAKE) build
	$(MAKE) up
	sleep 10
	@echo "${YELLOW}Checking services...${NC}"
	@if curl -f http://localhost:8080/health > /dev/null 2>&1; then \
		echo "${GREEN}✓ Backend health check passed${NC}"; \
	else \
		echo "${RED}✗ Backend health check failed${NC}"; \
	fi
	@if curl -f http://localhost:8888 > /dev/null 2>&1; then \
		echo "${GREEN}✓ Jupyter accessible${NC}"; \
	else \
		echo "${RED}✗ Jupyter not accessible${NC}"; \
	fi
	$(MAKE) down