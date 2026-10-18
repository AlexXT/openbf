//Decompile every function into one large C file (grep-friendly).
//Args: <output dir> [minSizeBytes]
//@category SWBF
import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.*;
import ghidra.program.model.listing.*;
import java.io.*;

public class ExportDecomp extends GhidraScript {
    public void run() throws Exception {
        String outDir = getScriptArgs().length > 0 ? getScriptArgs()[0] : "/tmp";
        long minSize = getScriptArgs().length > 1 ? Long.parseLong(getScriptArgs()[1]) : 0;

        DecompInterface ifc = new DecompInterface();
        DecompileOptions opts = new DecompileOptions();
        opts.setMaxPayloadMBytes(256);
        ifc.setOptions(opts);
        ifc.openProgram(currentProgram);

        PrintWriter fw = new PrintWriter(new BufferedWriter(new FileWriter(
                new File(outDir, currentProgram.getName() + "-decomp.c")), 1 << 20));
        FunctionManager fm = currentProgram.getFunctionManager();
        int n = 0, fail = 0, total = fm.getFunctionCount();
        for (Function f : fm.getFunctions(true)) {
            if (monitor.isCancelled()) break;
            if (f.getBody().getNumAddresses() < minSize) continue;
            fw.println("/* ==== " + f.getEntryPoint() + " " + f.getName() + " ==== */");
            DecompileResults res = ifc.decompileFunction(f, 90, monitor);
            if (res != null && res.decompileCompleted()) {
                fw.println(res.getDecompiledFunction().getC());
            } else {
                fw.println("/* DECOMPILE FAILED: " + (res != null ? res.getErrorMessage() : "null") + " */");
                fail++;
            }
            n++;
            if (n % 500 == 0) println("decompiled " + n + "/" + total);
        }
        fw.close();
        ifc.dispose();
        println("DECOMP done=" + n + " failed=" + fail);
    }
}
