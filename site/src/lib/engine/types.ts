export interface Letter {
	symbol: string;
	pronunciation: string;
	_type: 'vowel' | 'consonant';
}

export interface Alphabet {
	letters: Letter[];
}

export interface DictionaryEntry {
	root: string;
	definition: string;
	syntax?: SurfaceFormConfig;
}

export interface Dictionary {
	entries: DictionaryEntry[];
}

export interface SyllableAnalysis {
	spelling: string;
	pronunciation: string;
	stressed: boolean;
	graphemeStart: number;
	graphemeEnd: number;
}

export interface MorphemeAnalysis {
	kind: 'root';
	spelling: string;
	graphemeStart: number;
	graphemeEnd: number;
}

export interface WordAnalysis {
	spelling: string;
	root: string;
	morphemes: MorphemeAnalysis[];
	pronunciation: string;
	stressedPronunciation: string;
	rootStart: number;
	rootEnd: number;
	stressedSyllable: number;
	syllables: SyllableAnalysis[];
}

export interface SemanticAnalysis {
	inferred_type: string;
	canonical: string;
	explanation: string;
}

export interface SyntaxPolicy {
	frameOrder: string;
	freeOrder: boolean;
	scopeOpen: string;
	scopeClose: string;
	quoteOpen: string;
	quoteClose: string;
	explicitScope: string;
	quantifierScope: string;
	precedence: Record<string, number>;
	flattenSameOperator: boolean;
	lexicalRoots: number;
}

export interface SurfaceAnalysis {
	canonicalSurface: string;
	inferredType: string;
	canonicalSemantics: string;
	syntax: string;
}

export type DiagnosticLayer =
	| 'package'
	| 'phonology'
	| 'morphology'
	| 'lexical'
	| 'literal'
	| 'syntax'
	| 'discourse'
	| 'semantics'
	| 'pragmatics'
	| 'text_structure'
	| 'generation';

export interface AmbiguityCandidateView {
	id: string;
	ty: string;
	value: string;
	origin: string;
}

export interface WorkbenchDiagnostic {
	layer: DiagnosticLayer;
	message: string;
	candidates?: AmbiguityCandidateView[];
}

export interface PackageIdentity {
	name: string;
	version: string;
	revision: number;
}

export interface PackageManifest {
	package: PackageIdentity;
	provenance: { source: string; specification: string };
	modules: Record<string, string>;
	compatibility: { epoch: number };
	validation: {
		compatibility_corpus: string;
		adversarial_corpus: string;
		exhaustive_max_tokens: number;
	};
}

export interface SourceProvenance {
	path: string;
	digest: string;
	bytes: number;
}

export interface PackageProvenance {
	source: string;
	specification: string;
	fingerprint: string;
	sources: SourceProvenance[];
}

export interface PackageValidationReport {
	owned_forms: number;
	generated_asts: number;
	generated_surfaces: number;
	semantic_roundtrips: number;
	exhaustive_candidates: number;
	exhaustive_valid_surfaces: number;
	compatibility_entries: number;
	adversarial_entries: number;
}

export interface PackageWorkbenchInfo {
	manifest: PackageManifest;
	provenance: PackageProvenance;
	semantic_fingerprint: string;
	surface_fingerprint: string;
	validation: PackageValidationReport;
}

export type SurfaceFormConfig = { kind: string; [key: string]: unknown };

export interface WorkbenchDictionaryEntry {
	root: string;
	definition: string;
	syntax?: SurfaceFormConfig;
}

export interface WorkbenchMorpheme {
	kind: string;
	spelling: string;
	grapheme_start: number;
	grapheme_end: number;
}

export interface WorkbenchSyllable {
	spelling: string;
	pronunciation: string;
	stressed: boolean;
	grapheme_start: number;
	grapheme_end: number;
}

export interface WordWorkbenchAnalysis {
	source: string;
	root: string;
	definition: string;
	dictionary_entry: WorkbenchDictionaryEntry;
	semantic_origin?: string;
	pronunciation: string;
	stressed_pronunciation: string;
	morphemes: WorkbenchMorpheme[];
	syllables: WorkbenchSyllable[];
	package: PackageWorkbenchInfo;
}

export interface AstNodeView {
	kind: string;
	label: string;
	children: AstNodeView[];
}

export interface ScopeView {
	path: string;
	kind: string;
	operator: string;
}

export interface ResolvedBindingView {
	id?: string;
	ty: string;
	value: string;
	origin: string;
}

export interface ReferenceSlotView {
	placeholder: string;
	role: string;
	expected_type: string;
	source: string;
	resolution?: ResolvedBindingView;
}

export interface ContextSlotView {
	placeholder: string;
	surface: string;
	key: string;
	declared_type: string;
	expected_type: string;
	resolution?: ResolvedBindingView;
}

export interface AliasSlotView {
	placeholder: string;
	surface: string;
	alias: string;
	declared_type: string;
	expected_type: string;
	resolution?: ResolvedBindingView;
}

export interface SemanticNodeView {
	label: string;
	ty: string;
	origin?: string;
	children: { relation: string; node: SemanticNodeView }[];
}

export interface SurfaceWorkbenchAnalysis {
	source: string;
	canonical_surface: string;
	canonical_resolved_surface: string;
	inferred_type: string;
	surface_ast: AstNodeView;
	typed_template: string;
	references: ReferenceSlotView[];
	contexts: ContextSlotView[];
	aliases: AliasSlotView[];
	scopes: ScopeView[];
	semantic_ir: string;
	canonical_semantic_ir: string;
	semantic_explanation: SemanticNodeView;
	package_fingerprint: string;
}

export interface UtteranceWorkbenchAnalysis {
	surface: SurfaceWorkbenchAnalysis;
	pragmatics: { act: string; utterance: string; inferred_type: string };
}

export interface DiscourseStateView {
	scope: string;
	frame: string;
	referents: { id: string; ty: string; value: string; origin: string; scope: string; frame: string; shorthand: boolean }[];
	aliases: { surface: string; referent: string; ty: string; scope: string }[];
	history: { id: string; source: string; canonical_surface: string; act: string; semantics: string }[];
	active_commitments: { entry: string; content: string }[];
}

export interface DiscourseWorkbenchAnalysis {
	turns: {
		key: string;
		realization: string;
		source: string;
		before: DiscourseStateView;
		after: DiscourseStateView;
		events: {
			kind: string;
			id?: string;
			section: string;
			frame?: string;
			act?: string;
			source?: string;
			canonical_surface?: string;
			canonical_spoken?: string;
			canonical_written?: string;
			semantics?: string;
		}[];
	}[];
	final_state: DiscourseStateView;
	package: PackageWorkbenchInfo;
}

export interface GenerationWorkbenchAnalysis {
	input_semantics: string;
	canonical_semantics: string;
	inferred_type: string;
	surface_ast: AstNodeView;
	canonical_surface: string;
	roundtrip_semantics: string;
	roundtrip_verified: boolean;
	package_fingerprint: string;
}

export interface LiteralWorkbenchAnalysis {
	source: string;
	family: string;
	ty: string;
	semantic_canonical: string;
	canonical_written: string;
	canonical_spoken: string;
	realization: string;
	package_fingerprint: string;
}
