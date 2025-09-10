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

## Quick Start

### Prerequisites

- **Java 17** (JDK 17)
- **Maven 3.6+**
- **JavaFX 20.0.2** (SDK + JMODs)
- **Inno Setup 6** (for Windows installer)

### Environment Setup

1. Set `JAVA_HOME` to your JDK 17 installation
2. Set `FXJMODS` to your JavaFX JMODs directory:
   ```batch
   set FXJMODS=C:\javafx-jmods-20.0.2
   ```

---

## Build Commands

### 1. Compile and Test (Development)

```batch
# Clean and compile all modules
mvn -pl app -am -DskipTests clean package

# Run tests with hotkey debugging
.\test_hotkey.bat
```

### 2. Build Executable JAR

```batch
mvn -pl app -am -DskipTests clean package
```

**Output:** `app/target/file-organizer.jar`

### 3. Build Standalone Executable

```batch
# Use the provided compiler script
.\compiler.bat
```

**Output:** `dist/File Organizer/File Organizer.exe`

### 4. Build Windows Installer

```batch
# After running compiler.bat, the installer is automatically created
# Or manually:
"C:\Program Files (x86)\Inno Setup 6\ISCC.exe" ".\installer.iss"
```

**Output:** `FileOrganizer-Setup.exe`

---

## Testing Commands

### 1. Test Hotkey Functionality

```batch
# Test in development mode
.\test_hotkey.bat

# Test compiled executable
.\test_compiled.bat
```

### 2. Manual Testing

```batch
# Development mode
cd app
mvn javafx:run

# Compiled executable
"dist\File Organizer\File Organizer.exe"
```

**Test the hotkey:** Press `Ctrl+Space` to toggle the search window

---

## Development Workflow

### Quick Development Cycle

```batch
# 1. Compile
mvn -pl app -am -DskipTests clean package

# 2. Test hotkey
.\test_hotkey.bat

# 3. Build executable
.\compiler.bat

# 4. Test executable
.\test_compiled.bat
```

### Debug Mode

```batch
# Run with debug output
mvn -pl app javafx:run 2>&1 | findstr /i "hotkey"
```

---

## Troubleshooting

### Hotkey Not Working in Compiled Version

If hotkeys don't work in the compiled executable, ensure:

1. **JavaFX JMODs are properly configured:**

   ```batch
   set FXJMODS=C:\javafx-jmods-20.0.2
   ```

2. **All required modules are included:**

   - `java.logging` (required for JNA)
   - `java.desktop` (required for AWT)
   - `javafx.*` modules

3. **Check the logs:**
   ```batch
   type compiled_output.log
   ```

### Common Issues

- **"JavaFX runtime components are missing"** → Check JavaFX SDK installation
- **"NoClassDefFoundError: java/util/logging/Logger"** → Missing `java.logging` module
- **Hotkey registration fails** → Check Windows permissions and conflicting applications

# Notes

- Use `mvn javafx:run` during development (no need to mess with `--module-path`).

- Use `jpackage` **+ JMODs** to create distributable executables.

- Installer customization is handled in `installer.iss`.
