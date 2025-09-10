@echo off
echo Test du JAR avec logs d'erreur détaillés...
echo.
echo Compilation du JAR...
mvn -pl app clean package
echo.
echo Lancement du JAR avec capture des logs...
echo.
java -jar app/target/file-organizer.jar > jar_debug.log 2>&1
echo.
echo Sortie capturée dans jar_debug.log
echo.
echo Affichage des logs d'erreur hotkey:
findstr /i "hotkey" jar_debug.log
echo.
echo Appuyez sur une touche pour voir tous les logs...
pause
type jar_debug.log
