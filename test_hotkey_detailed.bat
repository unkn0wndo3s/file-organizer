@echo off
echo ========================================
echo TEST DETAILLE DU COMPORTEMENT HOTKEY
echo ========================================
echo.
echo L'application file-organizer doit etre en cours d'execution.
echo Regardez les logs dans la console pour voir le comportement.
echo.
echo ========================================
echo ETAPE 1: TEST DANS FENETRE NORMALE
echo ========================================
echo 1. Ouvrez l'Explorateur de fichiers
echo 2. Appuyez sur Ctrl+Space
echo 3. Verifiez que l'interface s'affiche
echo 4. Regardez les logs: [HOTKEY-DEBUG] Raccourci detecte!
echo.
echo ========================================
echo ETAPE 2: TEST DANS EDITEUR
echo ========================================
echo 1. Ouvrez Cursor (ou Notepad)
echo 2. Regardez les logs: [FOCUS] Focus sur un editeur de texte
echo 3. Regardez les logs: [HOTKEY-DEBUG] Desenregistrement de la hotkey
echo 4. Appuyez sur Ctrl+Space
echo 5. Verifiez que l'interface NE s'affiche PAS
echo 6. Verifiez que Ctrl+Space fait son action normale dans l'editeur
echo.
echo ========================================
echo ETAPE 3: RETOUR EN FENETRE NORMALE
echo ========================================
echo 1. Revenez a l'Explorateur de fichiers
echo 2. Regardez les logs: [FOCUS] Focus hors editeur de texte
echo 3. Regardez les logs: [HOTKEY-DEBUG] Enregistrement de la hotkey
echo 4. Appuyez sur Ctrl+Space
echo 5. Verifiez que l'interface s'affiche a nouveau
echo.
echo ========================================
echo LOGS A SURVEILLER
echo ========================================
echo - [FOCUS] Changement de focus vers: [nom]
echo - [FOCUS] Focus sur un editeur de texte / Focus hors editeur
echo - [HOTKEY-DEBUG] Desenregistrement de la hotkey (editeur detecte)
echo - [HOTKEY-DEBUG] Enregistrement de la hotkey (hors editeur)
echo - [HOTKEY-DEBUG] Raccourci detecte! (quand ca fonctionne)
echo.
pause