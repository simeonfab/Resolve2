@echo off
rem Double-click this file to open Resolve 2.
set "EXE=%~dp0target\release\resolve2.exe"
if not exist "%EXE%" set "EXE=%~dp0target\debug\resolve2.exe"
if not exist "%EXE%" (
  echo Resolve 2 hasn't been built yet. Run: cargo build
  pause
  exit /b 1
)
start "" "%EXE%"
