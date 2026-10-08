@echo off
setlocal
set PORT=COM6

if "%~1"=="" (
    echo usage: %~nx0 ^<file.elf^>
    exit /b 1
)
if not exist "%~1" (
    echo file not found: %~1
    exit /b 1
)

set HEX=%~dpn1.hex
avr-objcopy -O ihex -R .eeprom "%~1" "%HEX%" || exit /b 1
avrdude -c arduino -p atmega328p -P %PORT% -b 115200 -D -U flash:w:"%HEX%":i