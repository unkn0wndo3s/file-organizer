@echo off
echo ========================================
echo TEST DU COMPORTEMENT DE LA HOTKEY
echo ========================================
echo.
echo Instructions de test :
echo 1. L'application file-organizer doit etre en cours d'execution
echo 2. Testez Ctrl+Space dans differentes fenetres
echo.
echo ========================================
echo TEST 1: FENETRE NORMALE (Explorateur, etc.)
echo ========================================
echo - Ouvrez l'Explorateur de fichiers
echo - Appuyez sur Ctrl+Space
echo - La hotkey DEVRAIT fonctionner (interface s'affiche)
echo.
echo ========================================
echo TEST 2: EDITEUR DE TEXTE (Cursor, Notepad, etc.)
echo ========================================
echo - Ouvrez Cursor ou Notepad
echo - Appuyez sur Ctrl+Space
echo - La hotkey NE DEVRAIT PAS fonctionner
echo - (Ctrl+Space devrait faire son action normale dans l'editeur)
echo.
echo ========================================
echo TEST 3: RETOUR EN FENETRE NORMALE
echo ========================================
echo - Revenez a l'Explorateur de fichiers
echo - Appuyez sur Ctrl+Space
echo - La hotkey DEVRAIT refonctionner
echo.
echo ========================================
echo VERIFICATION DES LOGS
echo ========================================
echo Regardez les logs dans la console de l'application :
echo - [FOCUS] Changement de focus vers: [nom de la fenetre]
echo - [FOCUS] Focus sur un editeur de texte / Focus hors editeur de texte
echo - [HOTKEY-DEBUG] Désenregistrement de la hotkey (éditeur détecté)
echo - [HOTKEY-DEBUG] Enregistrement de la hotkey (hors éditeur)
echo.
pause
