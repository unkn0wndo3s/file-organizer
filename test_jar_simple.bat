@echo off
echo Test du JAR avec capture de sortie...
echo.
java -jar app/target/file-organizer.jar > jar_output.log 2>&1
echo.
echo Sortie capturée dans jar_output.log
echo.
echo Affichage des logs hotkey:
findstr /i "hotkey" jar_output.log
echo.
echo Appuyez sur une touche pour voir tous les logs...
pause
type jar_output.log
