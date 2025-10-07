@echo off
echo Test de la hotkey avec gestion du focus
echo ======================================
echo.
echo Ce script va tester le comportement de la hotkey selon le focus de l'editeur.
echo.
echo Instructions:
echo 1. Lancez l'application
echo 2. Ouvrez un editeur de texte (Notepad, VS Code, etc.)
echo 3. Observez les logs - la hotkey devrait etre desenregistree
echo 4. Changez de fenetre vers une autre application (pas d'editeur)
echo 5. Observez les logs - la hotkey devrait etre reenregistree
echo 6. Testez la hotkey dans les deux cas
echo.
echo Appuyez sur une touche pour continuer...
pause >nul

echo.
echo Compilation et execution...
mvn clean compile exec:java -Dexec.mainClass="fr.unkn0wndo3s.app.App" -Dexec.args="--debug" 2>&1 | findstr /i "hotkey\|focus\|edition"
