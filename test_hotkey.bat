@echo off
echo Test du raccourci clavier...
echo.
echo Lancement de l'application...
echo Appuyez sur Ctrl+Space ou Alt+F12 pour tester le raccourci
echo Appuyez sur Ctrl+C pour arrêter
echo.

mvn -pl app javafx:run 2>&1 | findstr /i "hotkey"
