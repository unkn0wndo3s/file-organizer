package fr.unkn0wndo3s.windows;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

public final class QuickAccessPinUtil {

    private QuickAccessPinUtil() {}

    // --- Public API ---

    /** Épingle dans Accès rapide. Marche pour dossier OU fichier. */
    public static boolean pin(Path item) {
        if (!isWindows() || item == null) return false;
        if (isPinned(item)) return true;           // déjà épinglé → on ne fait rien
        return invokeVerbOnParent(item, "pintohome");
    }

    /** Idempotent : n’épingle que si pas déjà épinglé. */
    public static boolean ensurePinned(Path item) {
        return pin(item);
    }


    /** Liste *toutes* les entrées visibles dans Quick Access (pinned + parfois “frequent”). */
    public static List<String> listQuickAccessItems() {
        String script = String.join(" ; ",
            "$sh = New-Object -ComObject Shell.Application",
            "$qa = $sh.Namespace('shell:::{679f85cb-0220-4080-b29b-5540cc05aab6}')",
            "$qa.Items() | ForEach-Object { $_.Path }"
        );
        try {
            return runPsCapture(script);
        } catch (Exception e) {
            return Collections.emptyList();
        }
    }

    /** True si le chemin exact est présent dans Quick Access (comparaison insensible à la casse). */
    public static boolean isPinned(Path item) {
        Path abs = toAbs(item);
        String want = abs.toString();
        for (String got : listQuickAccessItems()) {
            if (equalsPathCaseInsensitive(want, got)) return true;
        }
        return false;
    }

    /** RESET TOTAL : efface pins + historique et relance Explorer. */
    public static boolean resetQuickAccess() {
        if (!isWindows()) return false;
        boolean ok1 = deleteFileQuiet(findAutoDestPath());
        boolean ok2 = runCmd("taskkill /f /im explorer.exe") == 0;
        boolean ok3 = runCmd("start explorer.exe") == 0;
        return ok1 && ok2 && ok3;
    }

    // --- Impl ---

    /**
     * Applique le verbe shell sur l’élément en passant par son parent:
     * parent = Shell.NameSpace(dir), item = parent.ParseName(name), item.InvokeVerb(verb)
     * → fonctionne pour dossiers et fichiers.
     */
    private static boolean invokeVerbOnParent(Path item, String verb) {
        if (!isWindows()) return false;
        Objects.requireNonNull(item, "item");

        Path abs = toAbs(item);
        Path parent = abs.getParent();
        String name = (abs.getFileName() == null) ? null : abs.getFileName().toString();
        if (parent == null || name == null || name.isEmpty()) return false;

        String ps = String.join(" ; ",
            "$ErrorActionPreference='Stop'",
            "$sh = New-Object -ComObject Shell.Application",
            "$p  = " + psQuote(parent.toString()),
            "$n  = " + psQuote(name),
            "$f  = $sh.NameSpace($p)",
            "if ($f -eq $null) { exit 2 }",
            "$it = $f.ParseName($n)",
            "if ($it -eq $null) { exit 3 }",
            "$it.InvokeVerb(" + psQuote(verb) + ")"
        );

        int code = runPs(ps);
        // Explorer retourne parfois 1 sans stderr pour InvokeVerb → on tolère 0 et 1
        return code == 0 || code == 1;
    }

    // --- Utils ---

    private static boolean isWindows() {
        return System.getProperty("os.name","").toLowerCase().contains("win");
    }

    private static Path toAbs(Path p) {
        try { return p.toAbsolutePath().normalize(); } catch (Exception e) { return p; }
    }

    private static String psQuote(String s) {
        if (s == null) return "''";
        return "'" + s.replace("'", "''") + "'";
    }

    private static int runPs(String psScript) {
        ProcessBuilder pb = new ProcessBuilder(
            "powershell.exe", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", psScript
        );
        pb.redirectErrorStream(true);
        try {
            Process p = pb.start();
            // On draine la sortie sinon certains builds de Win11 bloquent tant que le flux n’est pas lu
            try (BufferedReader r = new BufferedReader(new InputStreamReader(p.getInputStream(), StandardCharsets.UTF_8))) {
                while (r.readLine() != null) { /* drain */ }
            }
            return p.waitFor();
        } catch (Exception e) {
            return -1;
        }
    }

    private static List<String> runPsCapture(String psScript) throws IOException, InterruptedException {
        ProcessBuilder pb = new ProcessBuilder(
            "powershell.exe", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", psScript
        );
        pb.redirectErrorStream(true);
        Process p = pb.start();
        List<String> lines = new ArrayList<>();
        try (BufferedReader r = new BufferedReader(new InputStreamReader(p.getInputStream(), StandardCharsets.UTF_8))) {
            String line;
            while ((line = r.readLine()) != null) lines.add(line);
        }
        p.waitFor();
        // Nettoyage basique des vides
        return lines.stream().filter(s -> s != null && !s.isBlank()).collect(Collectors.toList());
    }

    private static boolean equalsPathCaseInsensitive(String a, String b) {
        if (a == null || b == null) return false;
        return a.equalsIgnoreCase(b);
    }

    private static Path findAutoDestPath() {
        String up = System.getenv("USERPROFILE");
        if (up == null || up.isEmpty()) up = System.getProperty("user.home", "");
        return Paths.get(up, "AppData", "Roaming", "Microsoft", "Windows",
                         "Recent", "AutomaticDestinations", "f01b4d95cf55d32a.automaticDestinations-ms");
    }

    private static boolean deleteFileQuiet(Path p) {
        try { return Files.deleteIfExists(p); } catch (IOException e) { return false; }
    }

    /** Commande via cmd.exe. */
    private static int runCmd(String cmdLine) {
        ProcessBuilder pb = new ProcessBuilder("cmd.exe", "/d", "/c", cmdLine);
        pb.redirectErrorStream(true);
        try {
            Process p = pb.start();
            try (BufferedReader r = new BufferedReader(new InputStreamReader(p.getInputStream(), StandardCharsets.UTF_8))) {
                while (r.readLine() != null) { /* drain */ }
            }
            return p.waitFor();
        } catch (Exception e) {
            return -1;
        }
    }
}
