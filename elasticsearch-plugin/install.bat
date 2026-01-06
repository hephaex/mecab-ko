@echo off
REM MeCab-Ko Elasticsearch Plugin Installation Script for Windows
REM
REM Usage:
REM   install.bat [elasticsearch-home]
REM
REM Example:
REM   install.bat C:\elasticsearch
REM   install.bat %ES_HOME%

setlocal enabledelayedexpansion

REM Plugin info
set PLUGIN_NAME=mecab-ko-analyzer
set PLUGIN_VERSION=0.1.0

REM Parse arguments
set ES_HOME=%1
if "%ES_HOME%"=="" set ES_HOME=%ES_HOME%

if "%ES_HOME%"=="" (
    echo Error: Elasticsearch home directory not specified
    echo Usage: %0 [elasticsearch-home]
    echo Example: %0 C:\elasticsearch
    exit /b 1
)

if not exist "%ES_HOME%" (
    echo Error: Elasticsearch home directory not found: %ES_HOME%
    exit /b 1
)

REM Verify Elasticsearch installation
if not exist "%ES_HOME%\bin\elasticsearch.bat" (
    echo Error: elasticsearch.bat not found in %ES_HOME%\bin\
    exit /b 1
)

echo === MeCab-Ko Elasticsearch Plugin Installer ===
echo Elasticsearch home: %ES_HOME%
echo Plugin: %PLUGIN_NAME% v%PLUGIN_VERSION%
echo.

REM Check if plugin is already installed
set PLUGIN_DIR=%ES_HOME%\plugins\%PLUGIN_NAME%
if exist "%PLUGIN_DIR%" (
    echo Plugin already installed. Removing old version...
    rmdir /s /q "%PLUGIN_DIR%"
)

REM Build plugin if necessary
set PLUGIN_ZIP=%~dp0build\distributions\%PLUGIN_NAME%-%PLUGIN_VERSION%.zip
if not exist "%PLUGIN_ZIP%" (
    echo Plugin package not found. Building...
    cd /d %~dp0
    call gradlew.bat bundlePlugin
    if errorlevel 1 (
        echo Error: Failed to build plugin
        exit /b 1
    )
)

REM Install plugin using elasticsearch-plugin
echo Installing plugin...
if exist "%ES_HOME%\bin\elasticsearch-plugin.bat" (
    REM Use elasticsearch-plugin tool
    call "%ES_HOME%\bin\elasticsearch-plugin.bat" install "file:///%PLUGIN_ZIP:\=/%"
) else (
    REM Manual installation
    echo elasticsearch-plugin not found. Installing manually...

    mkdir "%PLUGIN_DIR%"

    REM Extract plugin (requires PowerShell)
    powershell -Command "Expand-Archive -Path '%PLUGIN_ZIP%' -DestinationPath '%PLUGIN_DIR%' -Force"
)

REM Verify installation
if exist "%PLUGIN_DIR%" (
    echo √ Plugin installed successfully
    echo.
    echo Installation details:
    echo   Plugin directory: %PLUGIN_DIR%
    echo   Version: %PLUGIN_VERSION%
    echo.

    REM Check native library
    set NATIVE_DIR=%PLUGIN_DIR%\native
    if exist "%NATIVE_DIR%" (
        echo √ Native libraries found
        dir "%NATIVE_DIR%"
    ) else (
        echo WARNING: Native libraries not found in %NATIVE_DIR%
        echo   Make sure to build native libraries first:
        echo     cd ..\rust
        echo     cargo build --release --features jni-bindings
    )

    echo.
    echo Installation complete!
    echo.
    echo Next steps:
    echo   1. Restart Elasticsearch:
    echo      net stop elasticsearch
    echo      net start elasticsearch
    echo      OR
    echo      %ES_HOME%\bin\elasticsearch.bat
    echo.
    echo   2. Verify plugin is loaded:
    echo      curl -X GET "localhost:9200/_cat/plugins?v"
    echo.
    echo   3. Test the analyzer:
    echo      curl -X POST "localhost:9200/_analyze" -H "Content-Type: application/json" -d "{\"analyzer\": \"mecab_ko\", \"text\": \"한국어 형태소 분석기\"}"
    echo.
) else (
    echo × Plugin installation failed
    exit /b 1
)

endlocal
