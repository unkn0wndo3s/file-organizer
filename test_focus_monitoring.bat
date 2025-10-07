@echo off
echo Test du monitoring du focus de la hotkey
echo =======================================
echo.
echo Ce script va tester le comportement de la hotkey avec monitoring du focus.
echo.
echo Instructions de test:
echo 1. Lancez l'application
echo 2. Observez les logs de demarrage - vous devriez voir l'etat initial
echo 3. Ouvrez un editeur de texte (Notepad, VS Code, etc.)
echo 4. Observez les logs - vous devriez voir "UNREGISTERING hotkey"
echo 5. Essayez d'utiliser la hotkey - AUCUN message ne devrait apparaître
echo 6. Changez de fenetre vers une autre application (pas d'editeur)
echo 7. Observez les logs - vous devriez voir "REGISTERING hotkey"
echo 8. Essayez d'utiliser la hotkey - vous devriez voir "Raccourci detecte!"
echo.
echo Appuyez sur une touche pour continuer...
pause >nul

echo.
echo Compilation et execution...
mvn clean compile exec:java -Dexec.mainClass="fr.unkn0wndo3s.app.App" -Dexec.args="--debug"
