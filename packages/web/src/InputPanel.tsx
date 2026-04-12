import React, { useRef, useEffect } from 'react';
import { Play, X } from 'lucide-react';
import { useStore } from './store';

export const InputPanel: React.FC = () => {
  const { input, setInput, parseInput, clear } = useStore();
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const handleInput = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const val = e.target.value;
    setInput(val);
    parseInput(val);
  };

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${Math.max(200, textareaRef.current.scrollHeight)}px`;
    }
  }, [input]);

  return (
    <div className="flex flex-col gap-2 p-4 bg-white dark:bg-slate-900 border-b border-slate-200 dark:border-slate-800">
      <div className="relative">
        <textarea
          ref={textareaRef}
          value={input}
          onChange={handleInput}
          placeholder="Paste your JSON-ish data here..."
          className="w-full min-h-[200px] max-h-[80vh] p-4 font-mono text-sm bg-slate-50 dark:bg-slate-950 border border-slate-300 dark:border-slate-700 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent outline-none resize-none transition-all text-slate-900 dark:text-slate-100"
        />
        <div className="absolute top-2 right-4 flex gap-2">
           <button
            onClick={() => parseInput(input)}
            className="p-2 bg-primary-600 hover:bg-primary-700 text-white rounded-md shadow-sm transition-colors cursor-pointer flex items-center justify-center"
            title="Parse (Ctrl+Enter)"
          >
            <Play size={18} fill="currentColor" />
          </button>
          <button
            onClick={clear}
            className="p-2 bg-slate-200 hover:bg-slate-300 dark:bg-slate-800 dark:hover:bg-slate-700 text-slate-700 dark:text-slate-300 rounded-md shadow-sm transition-colors cursor-pointer flex items-center justify-center"
            title="Clear"
          >
            <X size={18} />
          </button>
        </div>
      </div>
    </div>
  );
};
