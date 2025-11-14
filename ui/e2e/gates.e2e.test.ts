import { test, expect } from '@playwright/test';
import createClient from 'openapi-fetch';
import { paths } from '../src/lib/generated/api';

const client = createClient<paths>({
	baseUrl: 'http://localhost:9000/api'
});

test.beforeEach(async () => {
	await client.POST('/gates', {
		body: {
			group: 'some-group',
			service: 'some-service',
			environment: 'some-environment'
		}
	});
});

test.afterEach(async () => {
	await client.DELETE('/gates/{group}/{service}/{environment}', {
		params: {
			path: {
				group: 'some-group',
				service: 'some-service',
				environment: 'some-environment'
			}
		}
	});
});

test('should open and close gate', async ({ page }) => {
	// given
	await page.goto('/');

	// when (open group)
	await page.getByRole('tab', { name: 'some-group' }).click();

	// then
	await expect(page.getByRole('heading', { name: 'some-service' })).toBeVisible();
	await expect(page.getByRole('button', { name: 'closed' })).toBeVisible();

	// when (open gate)
	await page.getByRole('button', { name: 'closed' }).click();

	// then
	await expect(page.getByRole('button', { name: 'open' })).toBeVisible();

	// when (close gate)
	await page.getByRole('button', { name: 'open' }).click();

	// then
	await expect(page.getByRole('button', { name: 'closed' })).toBeVisible();
});

test('should add and remove comment', async ({ page }) => {
	// given
	await page.goto('/');

	// when (open group)
	await page.getByRole('tab', { name: 'some-group' }).click();

	// then
	await expect(page.getByRole('heading', { name: 'some-service' })).toBeVisible();

	// when (add comment)
	await page.getByRole('textbox').fill('Some new test comment.');
	await page.getByRole('button', { name: 'Send message' }).click();

	// then
	await expect(page.locator('.gate-comment-message')).toBeVisible();
	await expect(page.locator('.gate-comment-message')).toHaveText('Some new test comment.');

	// when (remove comment)
	await page.locator('.gate-comment-remove-button').click();

	// then
	await expect(page.locator('.gate-comment-message')).not.toBeInViewport();
});
