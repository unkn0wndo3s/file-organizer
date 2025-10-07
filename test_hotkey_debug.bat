@echo off
echo Test de debug de la hotkey
echo ==========================
echo.
echo Ce script va tester le comportement de la hotkey avec des logs detailles.
echo.
echo Instructions:
echo 1. Lancez l'application
echo 2. Ouvrez un editeur de texte (Notepad, VS Code, etc.)
echo 3. Observez les logs - vous devriez voir "Desenregistrement de la hotkey principale"
echo 4. Essayez d'utiliser la hotkey - AUCUN message ne devrait apparaître
echo 5. Changez de fenetre vers une autre application
echo 6. Observez les logs - vous devriez voir "Enregistrement de la hotkey principale"
echo 7. Essayez d'utiliser la hotkey - vous devriez voir "Raccourci detecte!"
echo.
echo Appuyez sur une touche pour continuer...
pause >nul

echo.
echo Compilation et execution...
mvn clean compile exec:java -Dexec.mainClass="fr.unkn0wndo3s.app.App" -Dexec.args="--debug"