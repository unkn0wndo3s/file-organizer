@echo off
echo Test de l'executable compile...
echo.
echo Lancement de l'application compilee...
echo Appuyez sur Ctrl+Space ou Alt+F12 pour tester le raccourci
echo Appuyez sur Ctrl+C pour arreter
echo.

cd /d "%~dp0"
"dist\File Organizer\File Organizer.exe" > compiled_output.log 2>&1
