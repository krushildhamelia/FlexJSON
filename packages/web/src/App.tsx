import * as Tabs from '@radix-ui/react-tabs';
import { 
  AlertTriangle, 
  AlertCircle, 
  Cpu,
  Layers,
  FileCode,
  Activity
} from 'lucide-react';
import { useStore } from './store';
import { InputPanel } from './InputPanel';
import { TreeViewer } from './TreeViewer';

function App() {
  const { parseResult, errorCount, warningCount } = useStore();

  return (
    <div className="flex flex-col h-screen bg-slate-50 dark:bg-slate-950 text-slate-900 dark:text-slate-100 overflow-hidden">
      {/* Top Bar */}
      <header className="h-14 shrink-0 flex items-center justify-between px-6 bg-white dark:bg-slate-900 border-b border-slate-200 dark:border-slate-800 shadow-sm z-10">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 bg-primary-600 rounded-lg flex items-center justify-center text-white shadow-md shadow-primary-500/20">
            <Cpu size={20} />
          </div>
          <div>
            <h1 className="text-lg font-bold tracking-tight text-slate-900 dark:text-white leading-none">
              FlexJSON <span className="text-primary-600 dark:text-primary-400 font-medium">Parser</span>
            </h1>
            <p className="text-[10px] text-slate-500 font-medium uppercase tracking-widest mt-0.5">WASM-Powered Precision</p>
          </div>
        </div>

        <div className="flex items-center gap-6">
          <div className="flex items-center gap-4 px-4 py-1.5 bg-slate-100 dark:bg-slate-800 rounded-full">
            <div className="flex items-center gap-1.5">
              <AlertCircle size={14} className={errorCount > 0 ? "text-red-500" : "text-slate-400"} />
              <span className={`text-xs font-bold ${errorCount > 0 ? "text-red-600 dark:text-red-400" : "text-slate-500"}`}>{errorCount}</span>
            </div>
            <div className="w-[1px] h-3 bg-slate-300 dark:bg-slate-600" />
            <div className="flex items-center gap-1.5">
              <AlertTriangle size={14} className={warningCount > 0 ? "text-amber-500" : "text-slate-400"} />
              <span className={`text-xs font-bold ${warningCount > 0 ? "text-amber-600 dark:text-amber-400" : "text-slate-500"}`}>{warningCount}</span>
            </div>
            <div className="w-[1px] h-3 bg-slate-300 dark:bg-slate-600" />
            <div className="flex items-center gap-1.5">
              <Layers size={14} className="text-primary-500" />
              <span className="text-xs font-bold text-primary-600 dark:text-primary-400">{parseResult?.fragments.length || 0}</span>
            </div>
          </div>
          
          {/* Removed Settings and GitHub buttons */}
        </div>
      </header>

      {/* Main Content */}
      <main className="flex-1 flex flex-col min-h-0 overflow-hidden">
        <InputPanel />

        <div className="flex-1 flex flex-col min-h-0 p-4">
          {parseResult && parseResult.fragments.length > 0 ? (
            <Tabs.Root defaultValue="frag-0" className="flex-1 flex flex-col min-h-0">
              <div className="flex items-center justify-between mb-2 gap-4">
                <div className="flex-1 min-w-0 overflow-hidden">
                  <Tabs.List 
                    className="flex gap-1 p-1 bg-slate-200/50 dark:bg-slate-800/50 rounded-lg overflow-x-auto no-scrollbar scroll-smooth cursor-grab active:cursor-grabbing select-none"
                    onMouseDown={(e) => {
                      const el = e.currentTarget;
                      const startX = e.pageX - el.offsetLeft;
                      const scrollLeft = el.scrollLeft;
                      
                      const onMouseMove = (e: MouseEvent) => {
                        e.preventDefault();
                        const x = e.pageX - el.offsetLeft;
                        const walk = (x - startX) * 2;
                        el.scrollLeft = scrollLeft - walk;
                      };
                      
                      const onMouseUp = () => {
                        window.removeEventListener('mousemove', onMouseMove);
                        window.removeEventListener('mouseup', onMouseUp);
                      };
                      
                      window.addEventListener('mousemove', onMouseMove);
                      window.addEventListener('mouseup', onMouseUp);
                    }}
                  >
                    {parseResult.fragments.map((_, i) => (
                      <Tabs.Trigger
                        key={i}
                        value={`frag-${i}`}
                        className="px-4 py-1.5 text-xs font-semibold rounded-md transition-all cursor-pointer shrink-0
                          data-[state=active]:bg-white dark:data-[state=active]:bg-slate-700 data-[state=active]:text-primary-600 dark:data-[state=active]:text-primary-400 data-[state=active]:shadow-sm
                          data-[state=inactive]:text-slate-500 hover:data-[state=inactive]:text-slate-700 dark:hover:data-[state=inactive]:text-slate-300"
                      >
                        Fragment {i + 1}
                      </Tabs.Trigger>
                    ))}
                  </Tabs.List>
                </div>
                
                <div className="flex items-center gap-2 text-[11px] font-medium text-slate-500 shrink-0">
                   <Activity size={12} />
                   <span>Interactive Mode Active</span>
                </div>
              </div>

              {parseResult.fragments.map((f, i) => (
                <Tabs.Content key={i} value={`frag-${i}`} className="flex-1 min-h-0 outline-none">
                  <TreeViewer fragment={f} />
                </Tabs.Content>
              ))}
            </Tabs.Root>
          ) : (
            <div className="flex-1 flex flex-col items-center justify-center text-slate-400 dark:text-slate-600 bg-white dark:bg-slate-900 rounded-xl border-2 border-dashed border-slate-200 dark:border-slate-800">
               <FileCode size={48} strokeWidth={1} className="mb-4 opacity-20" />
               <p className="text-sm font-medium">Ready to parse your JSON data</p>
               <p className="text-xs mt-1">Paste content above and click the parse icon to begin</p>
            </div>
          )}
        </div>
      </main>

      {/* Footer / Status Bar */}
      <footer className="h-8 shrink-0 flex items-center justify-between px-6 bg-slate-100 dark:bg-slate-900 border-t border-slate-200 dark:border-slate-800 text-[10px] text-slate-500 font-medium">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-1">
            <div className="w-1.5 h-1.5 rounded-full bg-green-500" />
            <span>WASM ENGINE: READY</span>
          </div>
          <span>MEMORY: 2.4MB</span>
        </div>
        <div className="flex items-center gap-4 uppercase tracking-tighter">
          <span>UTF-8</span>
          <span>LN 1, COL 1</span>
          <span className="text-primary-600 dark:text-primary-400 font-bold">V2.0.0-BETA</span>
        </div>
      </footer>
    </div>
  );
}

export default App;
