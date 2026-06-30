import { test, expect } from "@playwright/test";

test.describe("Signaling demo page", () => {
  test("navigates to signaling demo and shows heading", async ({ page }) => {
    await page.goto("/#demo/signaling");
    await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
    await expect(page.getByRole("heading", { level: 1 })).toContainText("Signaling");
  });

  test("signaling panel is visible with room join controls", async ({ page }) => {
    await page.goto("/#demo/signaling");
    await expect(page.getByRole("heading", { name: "WebRTC Signaling" })).toBeVisible();
    await expect(page.getByRole("textbox", { name: "Room ID" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Join" })).toBeVisible();
    // Join button disabled when room input is empty.
    await expect(page.getByRole("button", { name: "Join" })).toBeDisabled();
  });

  test("CRDT demo button is visible", async ({ page }) => {
    await page.goto("/#demo/signaling");
    await expect(page.getByRole("button", { name: "Run CRDT merge" })).toBeVisible();
  });

  test("join button enables when room ID is entered", async ({ page }) => {
    await page.goto("/#demo/signaling");
    await page.getByRole("textbox", { name: "Room ID" }).fill("test-room-001");
    await expect(page.getByRole("button", { name: "Join" })).toBeEnabled();
  });
});
