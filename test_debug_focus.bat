@echo off
echo Test de debug du monitoring du focus
echo ====================================
echo.
echo Ce script va afficher tous les logs de debug pour voir ce qui se passe.
echo.
echo Instructions:
echo 1. Lancez l'application
echo 2. Observez les logs de demarrage
echo 3. Ouvrez un editeur de texte (Notepad, VS Code, etc.)
echo 4. Observez les logs - vous devriez voir les classes de fenetre et la detection
echo 5. Changez de fenetre vers une autre application
echo 6. Observez les logs de changement d'etat
echo.
echo Appuyez sur une touche pour continuer...
pause >nul

echo.
echo Compilation et execution...
mvn clean compile exec:java -Dexec.mainClass="fr.unkn0wndo3s.app.App" -Dexec.args="--debug"
