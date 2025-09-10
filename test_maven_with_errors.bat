@echo off
echo Test avec Maven et capture des logs d'erreur...
echo.
mvn -pl app javafx:run > maven_output.log 2>&1
echo.
echo Sortie capturée dans maven_output.log
echo.
echo Affichage des logs hotkey:
findstr /i "hotkey" maven_output.log
echo.
echo Appuyez sur une touche pour voir tous les logs...
pause
type maven_output.log
