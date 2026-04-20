import { useMemo, useState, useCallback, useRef, useEffect } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import { 
  ChevronRight, 
  ChevronDown, 
  Package, 
  List, 
  Quote, 
  Hash, 
  CheckCircle2, 
  CircleOff,
  Minimize2,
  Maximize2,
  Copy,
  Check,
  Search,
  Filter
} from 'lucide-react';
import type { ParseNode, MemberNode, NormalizedFragment } from './store';

interface TreeRow {
  id: string;
  depth: number;
  label?: string;
  value?: any;
  node: ParseNode | MemberNode;
  hasChildren: boolean;
  isExpanded: boolean;
  type: 'object' | 'array' | 'string' | 'number' | 'boolean' | 'null' | 'member';
  matchesKey?: boolean;
  matchesValue?: boolean;
}

interface TreeViewerProps {
  fragment: NormalizedFragment;
}

const HighlightText: React.FC<{ text: string; highlight: string; className?: string }> = ({ text, highlight, className }) => {
  if (!highlight.trim()) return <span className={className}>{text}</span>;
  
  const parts = text.split(new RegExp(`(${highlight.replace(/[-[\]{}()*+?.,\\^$|#\s]/g, '\\$&')})`, 'gi'));
  return (
    <span className={className}>
      {parts.map((part, i) => 
        part.toLowerCase() === highlight.toLowerCase() ? (
          <mark key={i} className="bg-amber-200 dark:bg-amber-800/60 text-inherit rounded-sm px-0.5">
            {part}
          </mark>
        ) : (
          part
        )
      )}
    </span>
  );
};

export const TreeViewer: React.FC<TreeViewerProps> = ({ fragment }) => {
  const [expandedPaths, setExpandedPaths] = useState<Set<string>>(new Set(['root']));
  const [copied, setCopied] = useState(false);
  const [keySearch, setKeySearch] = useState('');
  const [valueSearch, setValueSearch] = useState('');
  const parentRef = useRef<HTMLDivElement>(null);

  // Auto-expand all paths when fragment changes
  useEffect(() => {
    const allPaths = new Set<string>();
    function collectPaths(node: ParseNode | MemberNode, path: string) {
      allPaths.add(path);
      if ('kind' in node) {
        if (node.kind === 'object') {
          node.children.forEach((child, i) => collectPaths(child, `${path}.c${i}`));
        } else if (node.kind === 'array') {
          node.elements.forEach((child, i) => collectPaths(child, `${path}.e${i}`));
        }
      } else {
        collectPaths(node.value, path);
      }
    }
    collectPaths(fragment.tree, 'root');
    setExpandedPaths(allPaths);
  }, [fragment.tree]);

  const toggleExpand = useCallback((path: string, recursive = false) => {
    setExpandedPaths(prev => {
      const next = new Set(prev);
      if (next.has(path)) {
        if (recursive) {
          for (const p of Array.from(next)) {
            if (p.startsWith(path)) next.delete(p);
          }
        } else {
          next.delete(path);
        }
      } else {
        next.add(path);
      }
      return next;
    });
  }, []);

  const expandAll = () => {
    const allPaths = new Set<string>();
    function collectPaths(node: ParseNode | MemberNode, path: string) {
      allPaths.add(path);
      if ('kind' in node) {
        if (node.kind === 'object') {
          node.children.forEach((child, i) => collectPaths(child, `${path}.c${i}`));
        } else if (node.kind === 'array') {
          node.elements.forEach((child, i) => collectPaths(child, `${path}.e${i}`));
        }
      } else {
        // MemberNode: we need to expand its value
        collectPaths(node.value, path);
      }
    }
    collectPaths(fragment.tree, 'root');
    setExpandedPaths(allPaths);
  };

  const copyToClipboard = () => {
    try {
      const prettyJson = JSON.stringify(JSON.parse(fragment.json), null, 2);
      navigator.clipboard.writeText(prettyJson);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (e) {
      console.error('Failed to format for copy', e);
      navigator.clipboard.writeText(fragment.json);
    }
  };

  const rows = useMemo(() => {
    const result: TreeRow[] = [];
    const searchActive = keySearch.trim() !== '' || valueSearch.trim() !== '';
    
    // Map to store if a path or any of its descendants matches the search
    const searchStatus = new Map<string, { matches: boolean; hasMatchInDescendants: boolean }>();

    function checkSearch(node: ParseNode | MemberNode, path: string, label?: string): { matches: boolean; hasMatchInDescendants: boolean } {
      let matches = false;
      let hasMatchInDescendants = false;

      // Check key match
      if (keySearch.trim() !== '' && label) {
        if (label.toLowerCase().includes(keySearch.toLowerCase())) {
          matches = true;
        }
      }

      // Check value match
      if (valueSearch.trim() !== '') {
        if ('kind' in node) {
          if (node.kind !== 'object' && node.kind !== 'array') {
            const valStr = node.kind === 'null' ? 'null' : String((node as any).value);
            if (valStr.toLowerCase().includes(valueSearch.toLowerCase())) {
              matches = true;
            }
          }
        }
      }

      // Check descendants
      if ('kind' in node) {
        if (node.kind === 'object') {
          node.children.forEach((child, i) => {
            const childStatus = checkSearch(child, `${path}.c${i}`, undefined); // MemberNode label is handled in the next branch
            if (childStatus.matches || childStatus.hasMatchInDescendants) hasMatchInDescendants = true;
          });
        } else if (node.kind === 'array') {
          node.elements.forEach((child, i) => {
            const childStatus = checkSearch(child, `${path}.e${i}`, `[${i}]`);
            if (childStatus.matches || childStatus.hasMatchInDescendants) hasMatchInDescendants = true;
          });
        }
      } else {
        // MemberNode
        const keyLabel = node.key.kind === 'string' ? node.key.value : node.key.raw;
        const childStatus = checkSearch(node.value, path, keyLabel);
        if (childStatus.matches || childStatus.hasMatchInDescendants) {
          hasMatchInDescendants = true;
          // If the child (value) matches or has matches, this MemberNode essentially "matches" because its key is part of the path
          // Actually, if the key matches, it's already caught by the label check above.
        }
      }

      searchStatus.set(path, { matches, hasMatchInDescendants });
      return { matches, hasMatchInDescendants };
    }

    if (searchActive) {
      checkSearch(fragment.tree, 'root', undefined);
    }

    function processNode(node: ParseNode | MemberNode, path: string, depth: number, label?: string) {
      const status = searchStatus.get(path);
      const isVisible = !searchActive || (status && (status.matches || status.hasMatchInDescendants));
      
      if (!isVisible) return;

      // In search mode, we force expansion if there are matching descendants
      const isExpanded = searchActive ? (status?.hasMatchInDescendants ?? false) : expandedPaths.has(path);
      
      let hasChildren = false;
      let value: any = null;
      let matchesKey = false;
      let matchesValue = false;

      if (searchActive && status?.matches) {
        // Determine if it was a key match or value match
        if (keySearch.trim() !== '' && label?.toLowerCase().includes(keySearch.toLowerCase())) {
          matchesKey = true;
        }
        if (valueSearch.trim() !== '') {
           if ('kind' in node && node.kind !== 'object' && node.kind !== 'array') {
             const valStr = node.kind === 'null' ? 'null' : String((node as any).value);
             if (valStr.toLowerCase().includes(valueSearch.toLowerCase())) {
               matchesValue = true;
             }
           }
        }
      }

      if ('kind' in node) {
        if (node.kind === 'object') {
          hasChildren = node.children.length > 0;
          result.push({ id: path, depth, label, node, hasChildren, isExpanded, type: 'object', matchesKey, matchesValue });
          if (isExpanded) {
            node.children.forEach((child, i) => {
              processNode(child, `${path}.c${i}`, depth + 1);
            });
          }
        } else if (node.kind === 'array') {
          hasChildren = node.elements.length > 0;
          result.push({ id: path, depth, label, node, hasChildren, isExpanded, type: 'array', matchesKey, matchesValue });
          if (isExpanded) {
            node.elements.forEach((child, i) => {
              processNode(child, `${path}.e${i}`, depth + 1, `[${i}]`);
            });
          }
        } else {
          // Leaf nodes
          value = node.kind === 'string' ? node.value : 
                  node.kind === 'number' ? node.value :
                  node.kind === 'boolean' ? node.value : null;
          result.push({ id: path, depth, label, node, hasChildren: false, isExpanded: false, type: node.kind, value, matchesKey, matchesValue });
        }
      } else {
        // MemberNode
        const keyLabel = node.key.kind === 'string' ? node.key.value : node.key.raw;
        processNode(node.value, path, depth, keyLabel);
      }
    }

    processNode(fragment.tree, 'root', 0);
    return result;
  }, [fragment.tree, expandedPaths, keySearch, valueSearch]);

  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 28,
    overscan: 20,
  });

  const handleKeyDown = (e: React.KeyboardEvent, row: TreeRow) => {
    if (e.key === ' ' || e.key === 'Enter') {
      e.preventDefault();
      toggleExpand(row.id, e.altKey);
    }
  };

  const getIcon = (type: TreeRow['type']) => {
    switch (type) {
      case 'object': return <Package size={14} className="text-purple-500" />;
      case 'array': return <List size={14} className="text-blue-500" />;
      case 'string': return <Quote size={14} className="text-green-600" />;
      case 'number': return <Hash size={14} className="text-orange-500" />;
      case 'boolean': return <CheckCircle2 size={14} className="text-indigo-500" />;
      case 'null': return <CircleOff size={14} className="text-slate-400" />;
      default: return null;
    }
  };

  return (
    <div className="flex flex-col h-full bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-lg overflow-hidden shadow-sm">
      {/* Toolbar */}
      <div className="flex flex-col bg-slate-50 dark:bg-slate-800/50 border-b border-slate-200 dark:border-slate-800">
        <div className="flex items-center justify-between px-4 py-2 border-b border-slate-200/50 dark:border-slate-700/50">
          <div className="flex items-center gap-4">
            <span className="text-xs font-semibold text-slate-500 uppercase tracking-wider">Tree Viewer</span>
            <div className="h-4 w-[1px] bg-slate-300 dark:bg-slate-700" />
            <div className="flex items-center gap-1 text-xs text-slate-600 dark:text-slate-400">
              <span className="font-mono">{rows.length}</span> rows
            </div>
          </div>
          <div className="flex items-center gap-2">
            <button onClick={expandAll} className="p-1.5 hover:bg-slate-200 dark:hover:bg-slate-700 rounded transition-colors text-slate-600 dark:text-slate-400" title="Expand All">
              <Maximize2 size={16} />
            </button>
             <button onClick={() => setExpandedPaths(new Set(['root']))} className="p-1.5 hover:bg-slate-200 dark:hover:bg-slate-700 rounded transition-colors text-slate-600 dark:text-slate-400" title="Collapse All">
              <Minimize2 size={16} />
            </button>
            <button 
              onClick={copyToClipboard} 
              className={`p-1.5 rounded transition-all flex items-center justify-center ${copied ? 'bg-green-100 text-green-600 dark:bg-green-900/30' : 'hover:bg-slate-200 dark:hover:bg-slate-700 text-slate-600 dark:text-slate-400'}`} 
              title={copied ? "Copied!" : "Copy JSON"}
            >
              {copied ? <Check size={16} /> : <Copy size={16} />}
            </button>
          </div>
        </div>

        {/* Search Bars */}
        <div className="flex items-center gap-2 px-3 py-1.5">
          <div className="flex-1 relative group">
            <Search size={12} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-slate-400 group-focus-within:text-primary-500 transition-colors" />
            <input 
              type="text" 
              placeholder="Filter keys..." 
              value={keySearch}
              onChange={(e) => setKeySearch(e.target.value)}
              className="w-full pl-8 pr-2 py-1.5 text-xs bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-md outline-none focus:ring-1 focus:ring-primary-500 focus:border-primary-500 transition-all"
            />
          </div>
          <div className="flex-1 relative group">
            <Filter size={12} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-slate-400 group-focus-within:text-indigo-500 transition-colors" />
            <input 
              type="text" 
              placeholder="Filter values..." 
              value={valueSearch}
              onChange={(e) => setValueSearch(e.target.value)}
              className="w-full pl-8 pr-2 py-1.5 text-xs bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-md outline-none focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 transition-all"
            />
          </div>
          {(keySearch || valueSearch) && (
            <button 
              onClick={() => { setKeySearch(''); setValueSearch(''); }}
              className="p-1 text-[10px] font-bold text-slate-400 hover:text-red-500 transition-colors"
            >
              CLEAR
            </button>
          )}
        </div>
      </div>

      {/* Tree content */}
      <div 
        ref={parentRef}
        className="flex-1 overflow-auto font-mono text-sm outline-none"
        tabIndex={0}
      >
        <div
          style={{
            height: `${virtualizer.getTotalSize()}px`,
            width: '100%',
            position: 'relative',
          }}
        >
          {virtualizer.getVirtualItems().map((virtualRow) => {
            const row = rows[virtualRow.index];
            return (
              <div
                key={row.id}
                data-index={virtualRow.index}
                ref={virtualizer.measureElement}
                onKeyDown={(e) => handleKeyDown(e, row)}
                onClick={(e) => row.hasChildren && toggleExpand(row.id, e.altKey)}
                tabIndex={0}
                className={`
                  absolute top-0 left-0 w-full flex items-center gap-2 px-2 py-0.5 cursor-pointer select-none
                  ${row.matchesKey || row.matchesValue ? 'bg-amber-50/50 dark:bg-amber-900/10' : 'hover:bg-primary-50 dark:hover:bg-primary-900/20'}
                  focus:bg-primary-100 dark:focus:bg-primary-900/40 outline-none
                  transition-colors group
                `}
                style={{
                  transform: `translateY(${virtualRow.start}px)`,
                }}
              >
                {/* Row indicator */}
                <div className="w-8 text-[10px] text-slate-400 dark:text-slate-600 text-right pr-2 border-r border-slate-100 dark:border-slate-800 mr-2 shrink-0">
                  {(virtualRow.index + 1).toString().padStart(2, '0')}
                </div>

                {/* Indentation */}
                <div style={{ width: `${row.depth * 1.5}rem` }} className="shrink-0 flex justify-end">
                   {row.depth > 0 && <div className="h-full border-l border-slate-200 dark:border-slate-800 ml-2" />}
                </div>

                {/* Expand icon */}
                <div className="w-4 flex items-center justify-center shrink-0">
                  {row.hasChildren ? (
                    row.isExpanded ? <ChevronDown size={14} className="text-slate-500" /> : <ChevronRight size={14} className="text-slate-500" />
                  ) : null}
                </div>

                {/* Node icon */}
                <div className="shrink-0">
                  {getIcon(row.type)}
                </div>

                {/* Label (key) */}
                {row.label && (
                  <span className="text-primary-700 dark:text-primary-400 font-medium shrink-0">
                    <HighlightText text={row.label} highlight={keySearch} />:
                  </span>
                )}

                {/* Value */}
                <div className="truncate text-slate-800 dark:text-slate-200">
                  {row.type === 'object' && <span className="text-slate-400 dark:text-slate-500">{row.isExpanded ? '{' : '{ ... }'}</span>}
                  {row.type === 'array' && <span className="text-slate-400 dark:text-slate-500">{row.isExpanded ? '[' : '[ ... ]'}</span>}
                  {row.type === 'string' && <span className="text-green-600 dark:text-green-400 break-all">"<HighlightText text={row.value} highlight={valueSearch} />"</span>}
                  {row.type === 'number' && <span className="text-orange-600 dark:text-orange-400"><HighlightText text={String(row.value)} highlight={valueSearch} /></span>}
                  {row.type === 'boolean' && <span className="text-indigo-600 dark:text-indigo-400"><HighlightText text={String(row.value)} highlight={valueSearch} /></span>}
                  {row.type === 'null' && <span className="text-slate-500 italic"><HighlightText text="null" highlight={valueSearch} /></span>}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};

