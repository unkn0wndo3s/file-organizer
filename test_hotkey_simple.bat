@echo off
echo Test simple de la hotkey
echo ========================
echo.
echo Compilation et execution...
mvn clean compile exec:java -Dexec.mainClass="fr.unkn0wndo3s.app.App" -Dexec.args="--debug" 2>&1 | findstr /i "hotkey\|focus\|edition\|ignor"