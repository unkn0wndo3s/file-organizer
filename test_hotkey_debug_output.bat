@echo off
echo Test des raccourcis clavier avec capture de sortie...
echo.
echo Lancement de l'application avec Maven...
echo.
mvn -pl app javafx:run > hotkey_debug.log 2>&1
echo.
echo Sortie capturée dans hotkey_debug.log
echo.
type hotkey_debug.log
pause
