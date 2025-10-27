@echo off
REM Windows batch script for serving WASM viewer

echo Setting up WASM viewer...
echo ========================
echo.

REM Check if config.js exists and setup env if needed
if not exist "www\config.js" (
    echo config.js not found, running setup...
    call setup-env.bat
)

REM Check if pkg exists in www
if not exist "www\pkg" (
    echo Copying pkg/ to www/...
    xcopy /E /I /Y pkg www\pkg
    echo [OK] Package copied
)

REM Check if output directory exists in www
if not exist "www\output" (
    echo Copying output/ to www/...
    xcopy /E /I /Y ..\data_pipeline\output www\output
    echo [OK] Output data copied
)

REM Check if output exists in data_pipeline
if not exist "..\data_pipeline\output\aggregated_data.json" (
    echo.
    echo WARNING: aggregated_data.json not found!
    echo    Please run the data pipeline first:
    echo    cd ..\data_pipeline ^&^& cargo run --bin data_pipeline
    echo.
)

echo.
echo Starting web server...
echo Open http://localhost:8000 in your browser
echo.

cd www
python -m http.server 8000

