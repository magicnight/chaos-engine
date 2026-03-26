// Generate abstract SVG category images for NewsPredict
// Usage: node scripts/gen-category-images.mjs

import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const OUT = join(import.meta.dirname, '..', 'public', 'images', 'categories');

const categories = {
  economics: {
    count: 4,
    colors: [['#00d4ff', '#0066ff', '#001a4d'], ['#0ea5e9', '#2563eb', '#1e1b4b'], ['#06b6d4', '#3b82f6', '#0f172a'], ['#00d4ff', '#6366f1', '#0c0a3e']],
    shapes: ['chart', 'bars', 'wave', 'grid'],
  },
  politics: {
    count: 3,
    colors: [['#f59e0b', '#ef4444', '#451a03'], ['#fbbf24', '#dc2626', '#1c1917'], ['#f97316', '#e11d48', '#18181b']],
    shapes: ['globe', 'columns', 'network'],
  },
  technology: {
    count: 4,
    colors: [['#8b5cf6', '#6366f1', '#0f0a2e'], ['#a78bfa', '#818cf8', '#1e1b4b'], ['#c084fc', '#7c3aed', '#0c0a3e'], ['#6366f1', '#3b82f6', '#0f172a']],
    shapes: ['circuit', 'hex', 'nodes', 'grid'],
  },
  geopolitics: {
    count: 3,
    colors: [['#ef4444', '#dc2626', '#1c0a0a'], ['#f87171', '#991b1b', '#18181b'], ['#fb923c', '#dc2626', '#1c1917']],
    shapes: ['radar', 'grid', 'wave'],
  },
  environment: {
    count: 3,
    colors: [['#22c55e', '#16a34a', '#052e16'], ['#4ade80', '#15803d', '#0f172a'], ['#34d399', '#059669', '#064e3b']],
    shapes: ['wave', 'hills', 'circles'],
  },
  health: {
    count: 3,
    colors: [['#ec4899', '#db2777', '#2e0519'], ['#f472b6', '#be185d', '#1c1917'], ['#fb7185', '#e11d48', '#18181b']],
    shapes: ['pulse', 'cross', 'circles'],
  },
  science: {
    count: 3,
    colors: [['#06b6d4', '#0891b2', '#042f2e'], ['#22d3ee', '#0e7490', '#0f172a'], ['#67e8f9', '#0891b2', '#164e63']],
    shapes: ['atom', 'orbit', 'constellation'],
  },
  entertainment: {
    count: 3,
    colors: [['#f97316', '#ea580c', '#1c0a00'], ['#fb923c', '#c2410c', '#18181b'], ['#fdba74', '#ea580c', '#1c1917']],
    shapes: ['spotlight', 'film', 'wave'],
  },
  sports: {
    count: 3,
    colors: [['#eab308', '#ca8a04', '#1c1a00'], ['#facc15', '#a16207', '#18181b'], ['#fde047', '#ca8a04', '#1c1917']],
    shapes: ['rings', 'field', 'motion'],
  },
  other: {
    count: 3,
    colors: [['#6b7280', '#4b5563', '#111827'], ['#9ca3af', '#374151', '#0f172a'], ['#d1d5db', '#6b7280', '#18181b']],
    shapes: ['abstract', 'dots', 'flow'],
  },
};

function makeShape(type, c1, c2, idx) {
  const shapes = {
    chart: `<polyline points="0,350 100,280 200,320 300,200 400,250 500,150 600,180 700,100 800,120" fill="none" stroke="${c1}" stroke-width="3" opacity="0.5"/>
      <polyline points="0,380 150,300 300,340 450,260 600,280 750,200 800,220" fill="none" stroke="${c2}" stroke-width="2" opacity="0.3"/>`,
    bars: Array.from({length:10}, (_,i) => {
      const h = 60 + Math.sin(i*0.8+idx)*80;
      return `<rect x="${i*80+5}" y="${400-h}" width="60" height="${h}" rx="4" fill="${c1}" opacity="${0.15+i*0.04}"/>`;
    }).join(''),
    wave: `<path d="M0,300 C100,${250-idx*20} 200,${350+idx*10} 300,280 C400,${210-idx*15} 500,${340+idx*10} 600,260 C700,${200-idx*10} 800,${320+idx*15} 800,300 L800,400 L0,400Z" fill="${c1}" opacity="0.15"/>
      <path d="M0,330 C150,${290-idx*10} 300,${370+idx*5} 450,310 C600,${270-idx*10} 750,${350+idx*5} 800,320 L800,400 L0,400Z" fill="${c2}" opacity="0.1"/>`,
    grid: Array.from({length:8}, (_,i) =>
      `<line x1="${i*120}" y1="0" x2="${i*120}" y2="400" stroke="${c1}" stroke-width="0.5" opacity="0.15"/>
       <line x1="0" y1="${i*60}" x2="800" y2="${i*60}" stroke="${c1}" stroke-width="0.5" opacity="0.15"/>`
    ).join(''),
    circuit: `<circle cx="200" cy="150" r="40" fill="none" stroke="${c1}" stroke-width="1.5" opacity="0.3"/>
      <circle cx="600" cy="250" r="60" fill="none" stroke="${c2}" stroke-width="1.5" opacity="0.2"/>
      <line x1="240" y1="150" x2="540" y2="250" stroke="${c1}" stroke-width="1" opacity="0.2"/>
      <circle cx="400" cy="200" r="4" fill="${c1}" opacity="0.4"/>
      <circle cx="300" cy="300" r="30" fill="none" stroke="${c1}" stroke-width="1" opacity="0.15"/>`,
    hex: Array.from({length:6}, (_,i) => {
      const cx = 100 + i*130, cy = 200 + (i%2)*80;
      return `<polygon points="${cx},${cy-40} ${cx+35},${cy-20} ${cx+35},${cy+20} ${cx},${cy+40} ${cx-35},${cy+20} ${cx-35},${cy-20}" fill="none" stroke="${c1}" stroke-width="1.5" opacity="${0.15+i*0.05}"/>`;
    }).join(''),
    nodes: Array.from({length:8}, (_,i) => {
      const cx=100+Math.cos(i)*250+idx*30, cy=200+Math.sin(i*1.3)*120;
      return `<circle cx="${cx}" cy="${cy}" r="${6+i*2}" fill="${c1}" opacity="${0.1+i*0.03}"/>`;
    }).join('') + `<line x1="100" y1="200" x2="400" y2="150" stroke="${c1}" stroke-width="0.8" opacity="0.15"/>`,
    globe: `<circle cx="400" cy="200" r="150" fill="none" stroke="${c1}" stroke-width="1.5" opacity="0.2"/>
      <ellipse cx="400" cy="200" rx="150" ry="50" fill="none" stroke="${c1}" stroke-width="1" opacity="0.15"/>
      <ellipse cx="400" cy="200" rx="60" ry="150" fill="none" stroke="${c2}" stroke-width="1" opacity="0.15"/>`,
    columns: `<rect x="150" y="150" width="60" height="200" rx="4" fill="${c1}" opacity="0.12"/>
      <rect x="250" y="100" width="60" height="250" rx="4" fill="${c1}" opacity="0.15"/>
      <rect x="350" y="130" width="60" height="220" rx="4" fill="${c2}" opacity="0.12"/>
      <rect x="450" y="170" width="60" height="180" rx="4" fill="${c1}" opacity="0.1"/>
      <rect x="550" y="120" width="60" height="230" rx="4" fill="${c2}" opacity="0.13"/>`,
    network: `<circle cx="200" cy="200" r="8" fill="${c1}" opacity="0.4"/><circle cx="400" cy="100" r="6" fill="${c1}" opacity="0.3"/>
      <circle cx="600" cy="200" r="10" fill="${c2}" opacity="0.35"/><circle cx="350" cy="300" r="7" fill="${c1}" opacity="0.3"/>
      <line x1="200" y1="200" x2="400" y2="100" stroke="${c1}" stroke-width="1" opacity="0.2"/>
      <line x1="400" y1="100" x2="600" y2="200" stroke="${c2}" stroke-width="1" opacity="0.2"/>
      <line x1="200" y1="200" x2="350" y2="300" stroke="${c1}" stroke-width="1" opacity="0.15"/>`,
    radar: `<polygon points="400,80 550,200 500,350 300,350 250,200" fill="none" stroke="${c1}" stroke-width="1.5" opacity="0.2"/>
      <polygon points="400,140 500,210 475,300 325,300 300,210" fill="${c1}" opacity="0.08"/>`,
    pulse: `<polyline points="50,200 150,200 200,100 250,300 300,200 350,200 400,120 450,280 500,200 600,200 650,140 700,260 750,200 800,200" fill="none" stroke="${c1}" stroke-width="2.5" opacity="0.3"/>`,
    cross: `<rect x="370" y="100" width="60" height="200" rx="8" fill="${c1}" opacity="0.12"/>
      <rect x="300" y="170" width="200" height="60" rx="8" fill="${c1}" opacity="0.12"/>`,
    circles: Array.from({length:5}, (_,i) =>
      `<circle cx="${150+i*130}" cy="${200+Math.sin(i)*50}" r="${30+i*10}" fill="none" stroke="${c1}" stroke-width="1.5" opacity="${0.1+i*0.04}"/>`
    ).join(''),
    atom: `<circle cx="400" cy="200" r="12" fill="${c1}" opacity="0.4"/>
      <ellipse cx="400" cy="200" rx="120" ry="40" fill="none" stroke="${c1}" stroke-width="1.5" opacity="0.2" transform="rotate(-30,400,200)"/>
      <ellipse cx="400" cy="200" rx="120" ry="40" fill="none" stroke="${c2}" stroke-width="1.5" opacity="0.2" transform="rotate(30,400,200)"/>
      <ellipse cx="400" cy="200" rx="120" ry="40" fill="none" stroke="${c1}" stroke-width="1.5" opacity="0.15"/>`,
    orbit: `<circle cx="400" cy="200" r="8" fill="${c1}" opacity="0.5"/>
      ${[80,130,180].map((r,i) => `<circle cx="400" cy="200" r="${r}" fill="none" stroke="${c1}" stroke-width="1" opacity="${0.1+i*0.05}"/>`).join('')}`,
    constellation: Array.from({length:12}, (_,i) => {
      const x=80+Math.cos(i*0.5)*300+idx*20, y=100+Math.sin(i*0.7)*150+idx*10;
      return `<circle cx="${x}" cy="${y}" r="${2+Math.random()*3}" fill="${c1}" opacity="${0.2+Math.random()*0.3}"/>`;
    }).join(''),
    spotlight: `<ellipse cx="400" cy="350" rx="300" ry="60" fill="${c1}" opacity="0.08"/>
      <polygon points="300,0 500,0 550,350 250,350" fill="url(#spot${idx})" opacity="0.12"/>
      <defs><linearGradient id="spot${idx}" x1="0" y1="0" x2="0" y2="1"><stop offset="0%" stop-color="${c1}" stop-opacity="0.5"/><stop offset="100%" stop-color="${c1}" stop-opacity="0"/></linearGradient></defs>`,
    film: `${[80,200,320].map(y => `<rect x="100" y="${y}" width="600" height="60" rx="4" fill="${c1}" opacity="0.08"/>`).join('')}`,
    rings: `<circle cx="300" cy="200" r="80" fill="none" stroke="${c1}" stroke-width="3" opacity="0.2"/>
      <circle cx="400" cy="200" r="80" fill="none" stroke="${c2}" stroke-width="3" opacity="0.2"/>
      <circle cx="500" cy="200" r="80" fill="none" stroke="${c1}" stroke-width="3" opacity="0.15"/>`,
    field: `<rect x="100" y="80" width="600" height="240" rx="8" fill="none" stroke="${c1}" stroke-width="1.5" opacity="0.15"/>
      <line x1="400" y1="80" x2="400" y2="320" stroke="${c1}" stroke-width="1" opacity="0.12"/>
      <circle cx="400" cy="200" r="60" fill="none" stroke="${c1}" stroke-width="1" opacity="0.12"/>`,
    motion: `<path d="M100,200 Q250,${100-idx*20} 400,200 Q550,${300+idx*20} 700,200" fill="none" stroke="${c1}" stroke-width="2.5" opacity="0.25"/>`,
    abstract: `<rect x="150" y="100" width="200" height="200" rx="20" fill="${c1}" opacity="0.08" transform="rotate(${15+idx*10},250,200)"/>
      <rect x="400" y="120" width="180" height="180" rx="20" fill="${c2}" opacity="0.06" transform="rotate(${-10+idx*5},490,210)"/>`,
    dots: Array.from({length:20}, (_,i) => {
      const x=60+i%5*160, y=60+Math.floor(i/5)*80;
      return `<circle cx="${x}" cy="${y}" r="${3+Math.sin(i)*2}" fill="${c1}" opacity="${0.08+Math.sin(i)*0.06}"/>`;
    }).join(''),
    flow: `<path d="M0,150 C200,100 300,250 500,200 C700,150 750,300 800,250" fill="none" stroke="${c1}" stroke-width="2" opacity="0.2"/>
      <path d="M0,250 C150,200 350,300 500,250 C650,200 700,350 800,300" fill="none" stroke="${c2}" stroke-width="1.5" opacity="0.15"/>`,
    hills: `<path d="M0,350 C100,250 200,300 300,250 C400,200 500,280 600,230 C700,180 750,250 800,220 L800,400 L0,400Z" fill="${c1}" opacity="0.12"/>`,
  };
  return shapes[type] || shapes.abstract;
}

function generateSVG(c1, c2, bg, shape, idx) {
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 400" width="800" height="400">
  <defs>
    <linearGradient id="bg${idx}" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="${bg}"/>
      <stop offset="50%" stop-color="${bg}"/>
      <stop offset="100%" stop-color="${c2}" stop-opacity="0.3"/>
    </linearGradient>
  </defs>
  <rect width="800" height="400" fill="url(#bg${idx})"/>
  ${makeShape(shape, c1, c2, idx)}
</svg>`;
}

let total = 0;
for (const [cat, cfg] of Object.entries(categories)) {
  const dir = join(OUT, cat);
  mkdirSync(dir, { recursive: true });
  for (let i = 0; i < cfg.count; i++) {
    const [c1, c2, bg] = cfg.colors[i];
    const svg = generateSVG(c1, c2, bg, cfg.shapes[i], i);
    const path = join(dir, `${i + 1}.svg`);
    writeFileSync(path, svg);
    total++;
  }
}
console.log(`Generated ${total} category images`);
