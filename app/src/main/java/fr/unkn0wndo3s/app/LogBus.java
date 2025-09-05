package fr.unkn0wndo3s.app;

import java.time.LocalTime;
import java.time.format.DateTimeFormatter;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.function.Consumer;

public final class LogBus {
    private static final List<Consumer<String>> listeners = new CopyOnWriteArrayList<>();
    private static final DateTimeFormatter HHMMSS = DateTimeFormatter.ofPattern("HH:mm:ss");

    private LogBus() {}

    public static void addListener(Consumer<String> l) { listeners.add(l); }
    public static void removeListener(Consumer<String> l) { listeners.remove(l); }

    public static void log(String line) {
        String ts = LocalTime.now().format(HHMMSS);
        String msg = "[" + ts + "] " + line;
        System.out.println(msg);
        System.out.flush();
        for (var l : listeners) {
            try { l.accept(msg); } catch (Throwable ignored) {}
        }
    }
}
