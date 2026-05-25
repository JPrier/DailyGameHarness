const GAME_ID = 'flag-fade';
const SPEC_VERSION = 'flag-fade.spec.v1';
const CANDIDATE_SET = 'countries-v1';
const MAX_GUESSES = 6;

const CANDIDATES = [
  country('jp', 'country:japan', 'Japan', ['Nippon', 'Nihon', 'JP', 'JPN', 'Japanese flag'], 'JP', 'JPN', 'Asia', ['white', 'red'], true, 'none', '2:3', ['central disc'], ['central-disc']),
  country('ca', 'country:canada', 'Canada', ['CA', 'CAN', 'Canadian flag'], 'CA', 'CAN', 'North America', ['red', 'white'], true, 'vertical', '1:2', ['vertical bands', 'leaf'], ['red-white']),
  country('bd', 'country:bangladesh', 'Bangladesh', ['BD', 'BGD', 'Bangladeshi flag'], 'BD', 'BGD', 'Asia', ['green', 'red'], true, 'none', '3:5', ['central disc'], ['central-disc']),
  country('id', 'country:indonesia', 'Indonesia', ['ID', 'IDN', 'Indonesian flag'], 'ID', 'IDN', 'Asia', ['red', 'white'], false, 'horizontal', '2:3', ['horizontal bicolor'], ['red-white']),
  country('kr', 'country:south-korea', 'South Korea', ['Republic of Korea', 'Korea', 'KR', 'KOR', 'ROK'], 'KR', 'KOR', 'Asia', ['white', 'red', 'blue', 'black'], true, 'none', '2:3', ['central emblem', 'corner marks'], ['central-emblem']),
  country('br', 'country:brazil', 'Brazil', ['Brasil', 'BR', 'BRA', 'Brazilian flag'], 'BR', 'BRA', 'South America', ['green', 'yellow', 'blue', 'white'], true, 'none', '7:10', ['diamond', 'central disc'], ['central-emblem']),
  country('pw', 'country:palau', 'Palau', ['PW', 'PLW', 'Palauan flag'], 'PW', 'PLW', 'Oceania', ['blue', 'yellow'], true, 'none', '5:8', ['central disc'], ['central-disc', 'blue-yellow']),
  country('fr', 'country:france', 'France', ['FR', 'FRA', 'French Republic', 'French flag'], 'FR', 'FRA', 'Europe', ['blue', 'white', 'red'], false, 'vertical', '2:3', ['vertical tricolor'], ['tricolor']),
  country('de', 'country:germany', 'Germany', ['Deutschland', 'DE', 'DEU', 'German flag'], 'DE', 'DEU', 'Europe', ['black', 'red', 'yellow'], false, 'horizontal', '3:5', ['horizontal tricolor'], ['tricolor']),
  country('us', 'country:united-states', 'United States', ['United States of America', 'USA', 'US', 'America', 'U.S.', 'U.S.A.'], 'US', 'USA', 'North America', ['red', 'white', 'blue'], true, 'horizontal', '10:19', ['horizontal stripes', 'canton', 'stars'], ['red-white-blue', 'red-white', 'canton']),
  country('gb', 'country:united-kingdom', 'United Kingdom', ['UK', 'GB', 'GBR', 'Great Britain', 'Britain', 'Union Jack'], 'GB', 'GBR', 'Europe', ['blue', 'white', 'red'], false, 'cross', '1:2', ['crosses', 'diagonal cross'], ['red-white-blue', 'cross', 'canton']),
  country('it', 'country:italy', 'Italy', ['Italia', 'IT', 'ITA', 'Italian flag'], 'IT', 'ITA', 'Europe', ['green', 'white', 'red'], false, 'vertical', '2:3', ['vertical tricolor'], ['tricolor']),
  country('es', 'country:spain', 'Spain', ['Espana', 'España', 'ES', 'ESP', 'Spanish flag'], 'ES', 'ESP', 'Europe', ['red', 'yellow'], true, 'horizontal', '2:3', ['horizontal triband', 'coat of arms'], ['red-yellow', 'horizontal-triband']),
  country('mx', 'country:mexico', 'Mexico', ['México', 'MX', 'MEX', 'Mexican flag'], 'MX', 'MEX', 'North America', ['green', 'white', 'red'], true, 'vertical', '4:7', ['vertical tricolor', 'central emblem'], ['tricolor', 'central-emblem']),
  country('in', 'country:india', 'India', ['Bharat', 'IN', 'IND', 'Indian flag'], 'IN', 'IND', 'Asia', ['orange', 'white', 'green', 'blue'], true, 'horizontal', '2:3', ['horizontal tricolor', 'central wheel'], ['horizontal-tricolor', 'central-emblem']),
  country('cn', 'country:china', 'China', ['People\'s Republic of China', 'PRC', 'CN', 'CHN', 'Chinese flag'], 'CN', 'CHN', 'Asia', ['red', 'yellow'], true, 'none', '2:3', ['stars', 'canton'], ['red-yellow', 'stars']),
  country('au', 'country:australia', 'Australia', ['AU', 'AUS', 'Australian flag'], 'AU', 'AUS', 'Oceania', ['blue', 'white', 'red'], true, 'canton', '1:2', ['canton', 'stars'], ['red-white-blue', 'canton', 'stars']),
  country('za', 'country:south-africa', 'South Africa', ['RSA', 'ZA', 'ZAF', 'South African flag'], 'ZA', 'ZAF', 'Africa', ['green', 'yellow', 'black', 'white', 'red', 'blue'], false, 'none', '2:3', ['y-shape', 'multicolor'], ['multicolor', 'red-white-blue']),
  country('ng', 'country:nigeria', 'Nigeria', ['NG', 'NGA', 'Nigerian flag'], 'NG', 'NGA', 'Africa', ['green', 'white'], false, 'vertical', '1:2', ['vertical triband'], ['green-white', 'vertical-triband', 'tricolor']),
  country('se', 'country:sweden', 'Sweden', ['Sverige', 'SE', 'SWE', 'Swedish flag'], 'SE', 'SWE', 'Europe', ['blue', 'yellow'], false, 'cross', '5:8', ['nordic cross'], ['blue-yellow', 'cross', 'nordic-cross']),
  country('no', 'country:norway', 'Norway', ['Norge', 'NO', 'NOR', 'Norwegian flag'], 'NO', 'NOR', 'Europe', ['red', 'white', 'blue'], false, 'cross', '8:11', ['nordic cross'], ['red-white-blue', 'cross', 'nordic-cross']),
  country('ch', 'country:switzerland', 'Switzerland', ['Swiss Confederation', 'CH', 'CHE', 'Swiss flag'], 'CH', 'CHE', 'Europe', ['red', 'white'], false, 'cross', '1:1', ['centered cross'], ['red-white', 'cross']),
  country('tr', 'country:turkey', 'Turkey', ['Türkiye', 'Turkiye', 'TR', 'TUR', 'Turkish flag'], 'TR', 'TUR', 'Asia', ['red', 'white'], true, 'none', '2:3', ['crescent', 'star'], ['red-white', 'stars']),
  country('ar', 'country:argentina', 'Argentina', ['AR', 'ARG', 'Argentine flag'], 'AR', 'ARG', 'South America', ['blue', 'white', 'yellow'], true, 'horizontal', '5:8', ['horizontal triband', 'central sun'], ['horizontal-triband', 'central-emblem']),
  country('ke', 'country:kenya', 'Kenya', ['KE', 'KEN', 'Kenyan flag'], 'KE', 'KEN', 'Africa', ['black', 'red', 'green', 'white'], true, 'horizontal', '2:3', ['horizontal tricolor', 'central shield'], ['horizontal-tricolor', 'central-emblem']),
];

function country(slug, entityId, canonicalName, aliases, iso2, iso3, continent, dominantColors, hasEmblem, stripeOrientation, aspectRatio, designFeatures, similarityGroups) {
  return {
    entityId,
    canonicalName,
    aliases,
    iso2,
    iso3,
    continent,
    slug,
    clues: { dominantColors, hasEmblem, stripeOrientation, aspectRatio, continent, designFeatures, similarityGroups },
  };
}

const NORMALIZED_CANDIDATES = new Map();
for (const candidate of CANDIDATES) {
  const keys = new Set([candidate.entityId, candidate.canonicalName, candidate.iso2, candidate.iso3, ...candidate.aliases].map(normalizeGuess));
  for (const key of keys) {
    if (!NORMALIZED_CANDIDATES.has(key)) NORMALIZED_CANDIDATES.set(key, []);
    NORMALIZED_CANDIDATES.get(key).push(candidate);
  }
}

export async function createRuntime() {
  return {
    contractVersion: 'daily-game-runtime.v1',
    validateContent,
    validatePuzzle,
    createInitialState,
    submitGuess,
    buildShareText,
  };
}

async function validateContent({ contentManifest }) {
  const errors = [];
  if (!contentManifest || contentManifest.schemaVersion !== 'daily-game-content-manifest.v1') {
    errors.push(error('schemaVersion', 'Manifest schemaVersion must be daily-game-content-manifest.v1.'));
  }
  if (contentManifest?.gameId !== GAME_ID) errors.push(error('gameId', 'Manifest gameId must be flag-fade.'));
  if (!Number.isInteger(contentManifest?.defaultMaxGuesses) || contentManifest.defaultMaxGuesses < 5 || contentManifest.defaultMaxGuesses > 7) {
    errors.push(error('defaultMaxGuesses', 'Flag Fade max guesses must be between 5 and 7.'));
  }
  if (contentManifest?.extension?.specVersion !== SPEC_VERSION) errors.push(error('extension.specVersion', 'Unsupported Flag Fade spec version.'));
  if (contentManifest?.extension?.candidateSet !== CANDIDATE_SET) errors.push(error('extension.candidateSet', 'Unsupported candidate set.'));
  if (contentManifest?.extension?.assetKind !== 'svg-or-png') errors.push(error('extension.assetKind', 'Asset kind must be svg-or-png.'));
  return result(errors);
}

async function validatePuzzle({ puzzle }) {
  const errors = [];
  if (!puzzle || puzzle.schemaVersion !== 'daily-game-puzzle.v1') errors.push(error('schemaVersion', 'Puzzle schemaVersion must be daily-game-puzzle.v1.'));
  if (puzzle?.gameId !== GAME_ID) errors.push(error('gameId', 'Puzzle gameId must be flag-fade.'));
  if (!nonEmpty(puzzle?.puzzleId)) errors.push(error('puzzleId', 'Puzzle id is required.'));
  if (!nonEmpty(puzzle?.seed)) errors.push(error('seed', 'Puzzle seed is required.'));
  if (!nonEmpty(puzzle?.display?.title)) errors.push(error('display.title', 'Puzzle title is required.'));
  if (!nonEmpty(puzzle?.display?.initialPrompt)) errors.push(error('display.initialPrompt', 'Initial prompt is required.'));

  const ext = puzzle?.extension;
  if (!ext || typeof ext !== 'object') {
    errors.push(error('extension', 'Puzzle extension is required.'));
    return result(errors);
  }

  const answer = ext.answer;
  if (!answer || typeof answer !== 'object') {
    errors.push(error('extension.answer', 'Answer is required.'));
  } else {
    if (!nonEmpty(answer.entityId)) errors.push(error('extension.answer.entityId', 'Answer entity id is required.'));
    if (!nonEmpty(answer.canonicalName)) errors.push(error('extension.answer.canonicalName', 'Canonical answer is required.'));
    if (!Array.isArray(answer.aliases) || answer.aliases.length < 1) errors.push(error('extension.answer.aliases', 'At least one accepted alias is required.'));
    if (!nonEmpty(answer.iso2)) errors.push(error('extension.answer.iso2', 'ISO2 code is required.'));
    if (!nonEmpty(answer.iso3)) errors.push(error('extension.answer.iso3', 'ISO3 code is required.'));
    if (!nonEmpty(answer.continent)) errors.push(error('extension.answer.continent', 'Continent is required.'));
  }

  if (ext.candidateSetId !== CANDIDATE_SET) errors.push(error('extension.candidateSetId', 'Puzzle candidate set is unsupported.'));
  if (!Array.isArray(ext.assetStages) || ext.assetStages.length < 5) {
    errors.push(error('extension.assetStages', 'At least five ordered visual stages are required.'));
  } else {
    ext.assetStages.forEach((stage, index) => {
      if (stage.stage !== index) errors.push(error(`extension.assetStages.${index}.stage`, 'Asset stages must be ordered from 0.'));
      if (!nonEmpty(stage.assetPath) || !/\.(svg|png)$/i.test(stage.assetPath)) errors.push(error(`extension.assetStages.${index}.assetPath`, 'Stage asset path must be an SVG or PNG.'));
      if (!Array.isArray(stage.reveals) || stage.reveals.length === 0) errors.push(error(`extension.assetStages.${index}.reveals`, 'Stage reveals are required.'));
    });
  }

  const clues = ext.clues ?? {};
  if (!Array.isArray(clues.dominantColors) || clues.dominantColors.length === 0) errors.push(error('extension.clues.dominantColors', 'Dominant colors cannot be empty.'));
  for (const key of ['hasEmblem', 'stripeOrientation', 'aspectRatio', 'continent']) {
    if (!(key in clues)) errors.push(error(`extension.clues.${key}`, `Missing clue ${key}.`));
  }
  if (!Array.isArray(ext.similarFlags) || ext.similarFlags.length === 0) errors.push(error('extension.similarFlags', 'Similar flag feedback configuration is required.'));
  return result(errors);
}

async function createInitialState({ contentManifest, puzzle, date }) {
  return {
    schemaVersion: 'daily-game-state.v1',
    gameId: GAME_ID,
    puzzleId: puzzle.puzzleId,
    date,
    status: 'in_progress',
    guessCount: 0,
    maxGuesses: contentManifest.defaultMaxGuesses ?? MAX_GUESSES,
    currentStage: 0,
    publicState: {
      history: [],
      revealedStage: 0,
      message: '',
    },
  };
}

async function submitGuess({ puzzle, state, input }) {
  if (!state || state.status !== 'in_progress') {
    return { state, evaluation: invalidEvaluation('Game is already complete.') };
  }

  const raw = typeof input?.value === 'string' ? input.value : '';
  const guessText = raw.trim();
  if (!guessText) {
    return { state: withMessage(state, 'Enter a country name.'), evaluation: invalidEvaluation('Enter a country name.') };
  }

  const matches = NORMALIZED_CANDIDATES.get(normalizeGuess(guessText)) ?? [];
  if (matches.length === 0) {
    return { state: withMessage(state, 'Unknown country. Try a recognized country name or ISO code.'), evaluation: invalidEvaluation('Unknown country. Try a recognized country name or ISO code.') };
  }
  if (matches.length > 1) {
    return { state: withMessage(state, `Be more specific: ${matches.map((m) => m.canonicalName).join(', ')}`), evaluation: invalidEvaluation('Country name is ambiguous.') };
  }

  const guess = matches[0];
  const answer = puzzle.extension.answer;
  const correct = guess.entityId === answer.entityId;
  const nextGuessCount = state.guessCount + 1;
  const nextStage = correct ? MAX_GUESSES - 1 : Math.min(MAX_GUESSES - 1, nextGuessCount);
  const status = correct ? 'won' : nextGuessCount >= state.maxGuesses ? 'lost' : 'in_progress';
  const feedback = correct ? [] : feedbackFor(guess, answer, puzzle.extension);
  const entry = {
    guess: guess.canonicalName,
    status: correct ? 'correct' : 'incorrect',
    feedback,
    stage: nextStage,
  };
  const publicState = {
    ...state.publicState,
    history: [...(state.publicState.history ?? []), entry],
    revealedStage: nextStage,
    message: correct ? 'Correct.' : status === 'lost' ? 'Game over.' : 'Incorrect.',
  };

  if (status !== 'in_progress') {
    publicState.reveal = revealFor(answer, puzzle.extension.assetStages[MAX_GUESSES - 1]?.assetPath);
  }

  const nextState = {
    ...state,
    status,
    guessCount: nextGuessCount,
    currentStage: nextStage,
    publicState,
  };
  return {
    state: nextState,
    evaluation: {
      outcome: correct ? 'correct' : 'incorrect',
      consumedGuess: true,
      message: publicState.message,
      feedback,
    },
  };
}

async function buildShareText({ state }) {
  const result = state.status === 'won' ? `${state.guessCount}/${state.maxGuesses}` : state.status === 'lost' ? `X/${state.maxGuesses}` : `${state.guessCount}/${state.maxGuesses}`;
  const rows = Array.from({ length: Math.max(1, state.guessCount) }, (_, index) => {
    if (state.status === 'won' && index === state.guessCount - 1) return '🟩';
    return '⬛';
  }).join('');
  return `🏳️ Flag Fade ${state.date}\nResult: ${result}\n${rows}`;
}

function feedbackFor(guess, answer, extension) {
  const answerClues = answer.clues ?? extension.clues ?? {};
  const guessClues = guess.clues ?? {};
  const exactColors = sameSet(guessClues.dominantColors, answerClues.dominantColors);
  const similar = areSimilar(guess, answer, extension.similarFlags);
  return [
    { key: 'continent', label: 'Same continent', kind: 'boolean', value: guess.continent === answer.continent, severity: guess.continent === answer.continent ? 'good' : 'bad' },
    { key: 'dominantColors', label: 'Exact colors', kind: 'set-exact', value: exactColors, severity: exactColors ? 'good' : 'bad' },
    { key: 'emblem', label: 'Emblem match', kind: 'boolean-match', value: guessClues.hasEmblem === answerClues.hasEmblem, severity: guessClues.hasEmblem === answerClues.hasEmblem ? 'good' : 'bad' },
    { key: 'stripeOrientation', label: 'Stripe layout', kind: 'enum-match', value: guessClues.stripeOrientation === answerClues.stripeOrientation, severity: guessClues.stripeOrientation === answerClues.stripeOrientation ? 'good' : 'bad' },
    { key: 'aspectRatio', label: 'Aspect ratio', kind: 'enum-match', value: guessClues.aspectRatio === answerClues.aspectRatio, severity: guessClues.aspectRatio === answerClues.aspectRatio ? 'good' : 'bad' },
    { key: 'similar', label: 'Similar flag family', kind: 'boolean', value: similar, severity: similar ? 'good' : 'neutral' },
  ];
}

function revealFor(answer, fullFlagAssetPath) {
  return {
    canonicalName: answer.canonicalName,
    iso2: answer.iso2,
    iso3: answer.iso3,
    continent: answer.continent,
    dominantColors: answer.clues?.dominantColors ?? [],
    fullFlagAssetPath,
  };
}

function areSimilar(guess, answer, configuredSimilarFlags = []) {
  const direct = configuredSimilarFlags.some((item) => {
    const ids = [item.entityId, ...(item.similarTo ?? [])];
    return ids.includes(guess.entityId) && ids.includes(answer.entityId);
  });
  if (direct) return true;
  return overlap(guess.clues?.similarityGroups, answer.clues?.similarityGroups).length > 0;
}

function overlap(left = [], right = []) {
  const rightSet = new Set(right);
  return left.filter((value) => rightSet.has(value));
}

function sameSet(left = [], right = []) {
  const leftSet = new Set(left);
  const rightSet = new Set(right);
  if (leftSet.size !== rightSet.size) return false;
  for (const value of leftSet) {
    if (!rightSet.has(value)) return false;
  }
  return true;
}

function normalizeGuess(value) {
  return String(value)
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase()
    .replace(/&/g, ' and ')
    .replace(/[^a-z0-9]+/g, ' ')
    .trim()
    .replace(/\s+/g, ' ');
}

function withMessage(state, message) {
  return { ...state, publicState: { ...state.publicState, message } };
}

function invalidEvaluation(message) {
  return { outcome: 'invalid', consumedGuess: false, message, feedback: [{ key: 'validation', label: 'Validation', kind: 'text', value: message, severity: 'warning' }] };
}

function nonEmpty(value) {
  return typeof value === 'string' && value.trim().length > 0;
}

function error(path, message) {
  return { code: 'invalid_flag_fade', path, message };
}

function result(errors) {
  return errors.length ? { ok: false, errors, warnings: [] } : { ok: true, warnings: [] };
}
