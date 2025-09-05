@echo off
cd /d "%~dp0"

call mvn -pl app -am -DskipTests clean package

set FXJMODS=C:\javafx-jmods-20.0.2

call "%JAVA_HOME%\bin\jpackage.exe" ^
  --type app-image ^
  --name "File Organizer" ^
  --input app\target ^
  --main-jar file-organizer.jar ^
  --main-class fr.unkn0wndo3s.app.Main ^
  --app-version 1.0.0 ^
  --vendor "Unkn0wndo3s" ^
  --icon fileorganizer.ico ^
  --module-path "%FXJMODS%;%JAVA_HOME%\jmods" ^
  --add-modules javafx.controls,javafx.fxml,javafx.graphics,javafx.base ^
  --dest dist ^
  --verbose

call "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" ".\installer.iss"

pause
