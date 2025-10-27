@echo off
REM Setup script to copy Mapbox token from .env to config.js

echo Setting up Mapbox token from .env...
echo ================================
echo.

REM Check if .env file exists
if not exist ".env" (
    echo [ERROR] .env file not found!
    echo.
    echo Creating .env from example...
    copy .env.example .env
    echo [OK] .env file created
    echo.
    echo WARNING: Please edit .env and add your Mapbox token
    echo    Get a token from: https://www.mapbox.com/
    echo.
    echo Then run this script again to generate config.js
    exit /b 1
)

REM Read MAPBOX_TOKEN from .env and create config.js
for /f "tokens=2 delims==" %%a in ('findstr "MAPBOX_TOKEN" .env') do set MAPBOX_TOKEN=%%a

if "%MAPBOX_TOKEN%"=="" (
    echo [ERROR] MAPBOX_TOKEN not set in .env file
    echo.
    echo Please edit .env and set MAPBOX_TOKEN=your_token_here
    exit /b 1
)

if "%MAPBOX_TOKEN%"=="YOUR_MAPBOX_ACCESS_TOKEN_HERE" (
    echo [ERROR] MAPBOX_TOKEN not set in .env file
    echo.
    echo Please edit .env and set MAPBOX_TOKEN=your_token_here
    exit /b 1
)

REM Create config.js
(
echo // This file is auto-generated from .env
echo // Do not edit manually - run setup-env.bat instead
echo const MAPBOX_ACCESS_TOKEN = '%MAPBOX_TOKEN%';
) > www\config.js

echo [OK] config.js generated with Mapbox token
echo.
echo Token loaded from .env file

