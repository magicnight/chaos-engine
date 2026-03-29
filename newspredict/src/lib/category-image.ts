const CATEGORY_IMAGE_COUNT: Record<string, number> = {
  economics: 3,
  politics: 3,
  technology: 3,
  geopolitics: 3,
  environment: 3,
  health: 3,
  science: 3,
  entertainment: 3,
  sports: 3,
  other: 3,
};

function simpleHash(str: string): number {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    hash = ((hash << 5) - hash + str.charCodeAt(i)) | 0;
  }
  return Math.abs(hash);
}

export function getCategoryImage(category: string, marketId: string): string {
  const cat = CATEGORY_IMAGE_COUNT[category] ? category : 'other';
  const count = CATEGORY_IMAGE_COUNT[cat];
  const index = (simpleHash(marketId) % count) + 1;
  return `/images/categories/${cat}/${index}.jpg`;
}
