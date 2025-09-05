# File Organizer

A desktop application built with **JavaFX** that organizes files and folders.  
It includes multiple modules: `core`, `ui`, `windows`, and `app`.

---

## Project Structure

```
file-organizer/
├── core/ # Core logic (file scanning, moving, initialization)
├── ui/ # JavaFX UI components
├── windows/ # Windows-specific utilities (Quick Access, hotkeys, etc.)
├── app/ # Main JavaFX application entry point
└── pom.xml # Parent Maven build file
```

---

## Dependencies

The project uses the following key dependencies:

- **Java 17** (JDK 17 or compatible with JavaFX 20+)
- **JavaFX 20.0.2** (base, controls, graphics, fxml)
- **Maven** (build & dependency management)
- **maven-shade-plugin** (to package a fat JAR)
- **javafx-maven-plugin** (for `mvn javafx:run`)
- **Inno Setup** (optional, to build a Windows installer)

### JavaFX Modules Used

- `javafx.base`
- `javafx.graphics`
- `javafx.controls`
- `javafx.fxml`

You need the **JavaFX SDK** (`.../lib` JARs) for development and the **JavaFX JMODs** (`.../jmods`) for packaging with `jpackage`.

---

## Useful Commands

### 1. Clean and install all modules

```
mvn -q -pl core,ui,windows,app -am clean install
```

# 2. Rebuild only the `app` module

```
mvn -q -pl app clean install
```

# 3. Run the application (JavaFX plugin)

```
cd app
mvn javafx:run
cd ..
```

# Build an Executable JAR

```
mvn -pl app -am -DskipTests clean package
```

This produces :

```
app/target/file-organizer.jar
```

Run it manually (requires JavaFX runtime on the module path):

```
java --module-path "C:\javafx-sdk-20.0.2\lib" ^
     --add-modules javafx.controls,javafx.fxml,javafx.graphics,javafx.base ^
     -jar app\target\file-organizer.jar
```

---

# Build a Standalone Executable (App Image)

Requires JavaFX **JMODs**:

```
set FXJMODS=C:\javafx-jmods-20.0.2
"%JAVA_HOME%\bin\jpackage.exe" ^
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
```

This generates:

```
dist/File Organizer/File Organizer.exe
```

# Build a Windows Installer

Install **Inno Setup** and compile `installer.iss`:

```
"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" ".\installer.iss"
```

This produces:

```
FileOrganizer-Setup.exe
```

# The installer includes an option to **start on Windows startup**.

---

# Typical Development Cycle

For quick tests:

```
clear
mvn -q -pl core,ui,windows,app -am clean install
mvn -q -pl app clean install
cd app
mvn javafx:run
cd ..
clear
```

# Notes

- Use `mvn javafx:run` during development (no need to mess with `--module-path`).

- Use `jpackage` **+ JMODs** to create distributable executables.

- Installer customization is handled in `installer.iss`.
