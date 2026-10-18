//Export function census + string cross-references after auto-analysis.
//Args: <output dir>
//@category SWBF
import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.*;
import ghidra.program.model.symbol.*;
import java.io.*;

public class ExportCensus extends GhidraScript {
    public void run() throws Exception {
        String outDir = getScriptArgs().length > 0 ? getScriptArgs()[0] : "/tmp";
        String base = currentProgram.getName();

        // 1. All functions: address, byte size, name
        PrintWriter fw = new PrintWriter(new BufferedWriter(
                new FileWriter(new File(outDir, base + "-functions.txt"))));
        FunctionManager fm = currentProgram.getFunctionManager();
        for (Function f : fm.getFunctions(true)) {
            fw.printf("%s %8d %s%n", f.getEntryPoint(), f.getBody().getNumAddresses(), f.getName());
        }
        fw.close();

        // 2. Defined strings with the functions that reference them
        PrintWriter sw = new PrintWriter(new BufferedWriter(
                new FileWriter(new File(outDir, base + "-stringrefs.txt"))));
        for (Data d : currentProgram.getListing().getDefinedData(true)) {
            Object vo = d.getValue();
            if (!(vo instanceof String)) continue;
            String val = ((String) vo).replace("\n", "\\n").replace("\r", "\\r").replace("\t", "\\t");
            StringBuilder refs = new StringBuilder();
            ReferenceIterator it = currentProgram.getReferenceManager().getReferencesTo(d.getAddress());
            int count = 0;
            while (it.hasNext() && count < 12) {
                Reference r = it.next();
                Function f = fm.getFunctionContaining(r.getFromAddress());
                refs.append(' ').append(f != null ? f.getName() + "@" + f.getEntryPoint() : "?@" + r.getFromAddress());
                count++;
            }
            sw.println(d.getAddress() + "\t" + val + "\t<-" + refs);
        }
        sw.close();

        println("CENSUS functions=" + fm.getFunctionCount());
    }
}
