@echo off
echo Test du JAR avec JavaFX...
echo.
echo Compilation du JAR...
mvn -pl app clean package
echo.
echo Lancement du JAR avec JavaFX...
echo.
mvn -pl app javafx:jlink
echo.
echo JAR créé avec JavaFX. Testez maintenant avec Ctrl+Space !
pause
