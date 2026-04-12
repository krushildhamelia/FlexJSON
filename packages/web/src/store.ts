import { create } from 'zustand';
import { parse } from 'core-wasm';

export interface ParseResult {
  fragments: NormalizedFragment[];
  warnings: ParseWarning[];
  errors: ParseError[];
}

export interface NormalizedFragment {
  index: number;
  source: Fragment;
  tree: ParseNode;
  json: string;
}

export interface Fragment {
  raw: string;
  startOffset: number;
  endOffset: number;
  startLine: number;
  startCol: number;
  type: string;
}

export type QuoteStyle = 'Double' | 'Single' | 'None';

export type ParseNode =
  | { kind: 'object'; children: MemberNode[]; raw: string }
  | { kind: 'array'; elements: ParseNode[]; raw: string }
  | { kind: 'string'; value: string; quoteStyle: QuoteStyle; raw: string }
  | { kind: 'number'; value: number; raw: string }
  | { kind: 'boolean'; value: boolean; raw: string }
  | { kind: 'null'; raw: string };

export interface MemberNode {
  key: ParseNode;
  separator: string;
  value: ParseNode;
  raw: string;
}

export interface ParseWarning {
  fragmentIndex: number;
  offset: number;
  line: number;
  col: number;
  code: string;
  message: string;
}

export type ParseError = 
  | { unmatchedBracket: string }
  | { unexpectedToken: string }
  | { invalidEscape: string }
  | { duplicateKey: string }
  | { maxFragmentSize: string }
  | { encoding: string };

export type FormatPreference = 'compact' | 'pretty' | 'canonical';

interface State {
  input: string;
  parseResult: ParseResult | null;
  formatPreference: FormatPreference;
  errorCount: number;
  warningCount: number;
}

interface Actions {
  setInput: (input: string) => void;
  setFormatPreference: (pref: FormatPreference) => void;
  parseInput: (text: string) => void;
  clear: () => void;
}

export const useStore = create<State & Actions>((set) => ({
  input: '',
  parseResult: null,
  formatPreference: 'pretty',
  errorCount: 0,
  warningCount: 0,

  setInput: (input) => set({ input }),
  setFormatPreference: (pref) => set({ formatPreference: pref }),
  parseInput: (text) => {
    try {
      const result = parse(text) as ParseResult;
      set({
        input: text,
        parseResult: result,
        errorCount: result.errors.length,
        warningCount: result.warnings.length,
      });
    } catch (e) {
      console.error('Parse failed', e);
    }
  },
  clear: () => set({ input: '', parseResult: null, errorCount: 0, warningCount: 0 }),
}));
