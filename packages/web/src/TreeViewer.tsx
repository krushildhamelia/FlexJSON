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
  Check
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
}

interface TreeViewerProps {
  fragment: NormalizedFragment;
}

export const TreeViewer: React.FC<TreeViewerProps> = ({ fragment }) => {
  const [expandedPaths, setExpandedPaths] = useState<Set<string>>(new Set(['root']));
  const [copied, setCopied] = useState(false);
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
    
    function processNode(node: ParseNode | MemberNode, path: string, depth: number, label?: string) {
      const isExpanded = expandedPaths.has(path);
      let hasChildren = false;
      let value: any = null;

      if ('kind' in node) {
        if (node.kind === 'object') {
          hasChildren = node.children.length > 0;
          result.push({ id: path, depth, label, node, hasChildren, isExpanded, type: 'object' });
          if (isExpanded) {
            node.children.forEach((child, i) => {
              // Using index in path to ensure uniqueness
              processNode(child, `${path}.c${i}`, depth + 1);
            });
          }
        } else if (node.kind === 'array') {
          hasChildren = node.elements.length > 0;
          result.push({ id: path, depth, label, node, hasChildren, isExpanded, type: 'array' });
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
          result.push({ id: path, depth, label, node, hasChildren: false, isExpanded: false, type: node.kind, value });
        }
      } else {
        // MemberNode
        const keyLabel = node.key.kind === 'string' ? node.key.value : node.key.raw;
        processNode(node.value, path, depth, keyLabel);
      }
    }

    processNode(fragment.tree, 'root', 0);
    return result;
  }, [fragment.tree, expandedPaths]);

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
      <div className="flex items-center justify-between px-4 py-2 bg-slate-50 dark:bg-slate-800/50 border-b border-slate-200 dark:border-slate-800">
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
                  hover:bg-primary-50 dark:hover:bg-primary-900/20 focus:bg-primary-100 dark:focus:bg-primary-900/40 outline-none
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
                    {row.label}:
                  </span>
                )}

                {/* Value */}
                <div className="truncate text-slate-800 dark:text-slate-200">
                  {row.type === 'object' && <span className="text-slate-400 dark:text-slate-500">{row.isExpanded ? '{' : '{ ... }'}</span>}
                  {row.type === 'array' && <span className="text-slate-400 dark:text-slate-500">{row.isExpanded ? '[' : '[ ... ]'}</span>}
                  {row.type === 'string' && <span className="text-green-600 dark:text-green-400 break-all">"{row.value}"</span>}
                  {row.type === 'number' && <span className="text-orange-600 dark:text-orange-400">{row.value}</span>}
                  {row.type === 'boolean' && <span className="text-indigo-600 dark:text-indigo-400">{String(row.value)}</span>}
                  {row.type === 'null' && <span className="text-slate-500 italic">null</span>}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};
