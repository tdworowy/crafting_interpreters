# Makefile for interpreter_in_python
# Cross-platform: Linux, macOS, Windows (cmd / PowerShell / Git Bash / WSL)

.PHONY: help install dev-install build clean format lint check test coverage mypy ruff up ci fix
.DEFAULT_GOAL := help

# ────────────────────────────────────────
#  OS detection & tool paths
# ────────────────────────────────────────

ifeq ($(OS),Windows_NT)
    PYTHON := python
    VENV_BIN := .venv\Scripts
    PIP := $(VENV_BIN)\pip.exe
    PYTEST := $(VENV_BIN)\pytest.exe
    RUFF := $(VENV_BIN)\ruff.exe
    BLACK := $(VENV_BIN)\black.exe
    MYPY := $(VENV_BIN)\mypy.exe
    RM_RF := rmdir /s /q
    RM_FILE := del /q /f
    NULL := >nul 2>&1
    WIN := 1
else
    PYTHON := python3
    VENV_BIN := .venv/bin
    PIP := $(VENV_BIN)/pip
    PYTEST := $(VENV_BIN)/pytest
    RUFF := $(VENV_BIN)/ruff
    BLACK := $(VENV_BIN)/black
    MYPY := $(VENV_BIN)/mypy
    RM_RF := rm -rf
    RM_FILE := rm -f
    NULL :=
    WIN :=
endif

# ────────────────────────────────────────
#  Help (fully cross-platform)
# ────────────────────────────────────────

help: ## Show available targets
	@echo ""
	@echo "Available targets:"
ifdef WIN
	@echo "  install       →  Install package (editable) + minimal dependencies"
	@echo "  dev-install   →  Install package + all dev dependencies (black, ruff, mypy, pytest, etc.)"
	@echo "  build         →  Build source and wheel distributions"
	@echo "  clean         →  Remove build artifacts and caches"
	@echo "  format        →  Auto-format code with black + ruff (fix mode)"
	@echo "  ruff          →  Run ruff lint + format check"
	@echo "  mypy          →  Run mypy type checker"
	@echo "  lint          →  Run ruff + mypy"
	@echo "  check         →  format + lint + test (good for CI/pre-push)"
	@echo "  test          →  Run tests with pytest"
	@echo "  test-fast     →  Quick tests without coverage"
	@echo "  coverage      →  Tests + HTML coverage report"
	@echo "  up            →  Full cycle: dev-install → format → check → test"
	@echo "  ci            →  check + test (CI-friendly)"
	@echo "  fix           →  Alias for format"
else
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
endif
	@echo ""

# ────────────────────────────────────────
#  Installation
# ────────────────────────────────────────

install: ## Install package in editable mode + minimal deps
	$(PYTHON) -m venv .venv
	$(PYTHON) -m pip install --upgrade pip setuptools wheel
	$(PIP) install -e .

dev-install: ## Install package + all dev dependencies
	$(PYTHON) -m venv .venv
	$(PYTHON) -m pip install --upgrade pip setuptools wheel
	$(PIP) install -e ".[dev]"

# ────────────────────────────────────────
#  Build & Clean
# ────────────────────────────────────────

build: clean ## Build sdist + wheel
	$(PYTHON) -m build

clean: ## Remove build artifacts, caches, etc.
	-$(RM_RF) build dist *.egg-info .eggs .pytest_cache .ruff_cache .mypy_cache htmlcov .coverage coverage.xml $(NULL)
	-$(RM_RF) src\__pycache__ tests\__pycache__ $(NULL) 2>nul
	-$(RM_FILE) *.pyc *.pyo *~ $(NULL)

# ────────────────────────────────────────
#  Formatting & Linting
# ────────────────────────────────────────

format: ## Auto-format code with black + ruff fix
	$(PYTHON) -m black src tests
	$(PYTHON) -m ruff check --fix src tests

ruff: ## Ruff lint + format check
	$(PYTHON) -m ruff check src tests
	$(PYTHON) -m ruff format --check src tests

mypy: ## Run mypy type checker
	$(PYTHON) -m mypy src tests

lint: ruff mypy ## Run all static checks (non-fixing)

check: format lint test ## Format → lint → test (good for CI/pre-push)

# ────────────────────────────────────────
#  Testing
# ────────────────────────────────────────

test: ## Run tests
	$(PYTEST) -rasvv

test-fast: ## Quick test run, no coverage
	$(PYTEST) -q --no-cov

coverage: ## Tests + coverage report
	$(PYTEST)  tests--cov=interpreter_in_python --cov-report=term-missing --cov-report=html

# ────────────────────────────────────────
#  Aliases / shortcuts
# ────────────────────────────────────────

up: dev-install format check test ## Full quality cycle
ci: check test ## CI-friendly
fix: format ## Alias for format