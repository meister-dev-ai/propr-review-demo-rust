import { expect, test } from '@playwright/test';

test('home page shows ordered navigation', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByRole('heading', { level: 1, name: 'Review-ready demo content' })).toBeVisible();

  const links = page.getByRole('navigation', { name: 'Primary navigation' }).getByRole('link');
  await expect(links).toHaveText(['Propr Review Demo', 'Blog', 'About']);
  await expect(links.nth(0)).toHaveAttribute('href', '/');
  await expect(links.nth(1)).toHaveAttribute('href', '/blog/');
  await expect(links.nth(2)).toHaveAttribute('href', '/about/');
});

test('about page renders project details', async ({ page }) => {
  await page.goto('/about/');

  await expect(page.getByRole('heading', { level: 1, name: 'About this project' })).toBeVisible();
  await expect(page.getByText('content lives in')).toBeVisible();
  await expect(page.getByText('markdown is compiled into static HTML at build time')).toBeVisible();
});

test('blog listing shows articles in date order', async ({ page }) => {
  await page.goto('/blog/');

  await expect(page.getByRole('heading', { level: 1, name: 'Latest articles' })).toBeVisible();
  const articleLinks = page.locator('.article-card h2 a');
  await expect(articleLinks).toHaveText(['Welcome to the Demo', 'Reviewing Pull Requests Effectively']);
});

test('article page renders content and back link', async ({ page }) => {
  await page.goto('/blog/welcome-to-the-demo/');

  await expect(page.getByRole('heading', { level: 1, name: 'Welcome to the demo' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Back to Blog' })).toHaveAttribute('href', '/blog/');
  await expect(page.getByText('Write markdown, add frontmatter')).toBeVisible();
});

test('older article remains second in listing', async ({ page }) => {
  await page.goto('/blog/');

  const articleLinks = page.locator('.article-card h2 a');
  await expect(articleLinks.nth(1)).toHaveText('Reviewing Pull Requests Effectively');
  await articleLinks.nth(1).click();
  await expect(page).toHaveURL(/\/blog\/reviewing-pull-requests-effectively\/$/);
});
