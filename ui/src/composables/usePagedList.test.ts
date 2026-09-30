import { describe, expect, test } from "bun:test";
import { nextTick, ref } from "vue";
import type { ApiResult } from "@/services/ApiService";
import { useNotify } from "@/composables/useNotify";
import { usePagedList } from "./usePagedList";

const PAGES = 3;

const pageOf = (page: number): ApiResult<string> => ({
  status: "success",
  count: PAGES * 2,
  page,
  pages: PAGES,
  result: [`item ${page}.1`, `item ${page}.2`],
});

// Lets pending promises (the fetch and the code awaiting it) run to completion.
const settle = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 0));

describe("usePagedList", () => {
  test("loads the requested page", async () => {
    // arrange
    const requested = ref(2);

    // act
    const list = usePagedList(async (p) => pageOf(p), requested, "failed");
    await settle();

    // assert
    expect(list.items.value).toEqual(["item 2.1", "item 2.2"]);
    expect(list.page.value).toBe(2);
    expect(list.pages.value).toBe(PAGES);
    expect(list.loading.value).toBe(false);
    expect(list.failed.value).toBe(false);
  });

  test("is loading until the page arrives", () => {
    // arrange
    const requested = ref(1);

    // act
    const list = usePagedList(() => new Promise<ApiResult<string>>(() => {}), requested, "failed");

    // assert
    expect(list.loading.value).toBe(true);
    expect(list.items.value).toEqual([]);
  });

  test("follows page changes", async () => {
    // arrange
    const requested = ref(1);
    const list = usePagedList(async (p) => pageOf(p), requested, "failed");
    await settle();

    // act
    requested.value = 3;
    await nextTick();
    await settle();

    // assert
    expect(list.page.value).toBe(3);
    expect(list.items.value).toEqual(["item 3.1", "item 3.2"]);
  });

  test("refresh reloads the current page", async () => {
    // arrange
    const requested = ref(2);
    const fetched: Array<number> = [];
    const list = usePagedList(
      async (p) => {
        fetched.push(p);
        return pageOf(p);
      },
      requested,
      "failed"
    );
    await settle();

    // act
    await list.refresh();

    // assert
    expect(fetched).toEqual([2, 2]);
  });

  test("marks the list failed and notifies when loading fails", async () => {
    // arrange
    const requested = ref(1);
    let fail = false;
    const list = usePagedList(
      async (p) => {
        if (fail) {
          throw new Error("boom");
        }
        return pageOf(p);
      },
      requested,
      "Не удалось загрузить"
    );
    await settle();
    fail = true;
    const notify = useNotify();

    // act
    await list.refresh();

    // assert
    expect(list.failed.value).toBe(true);
    expect(list.loading.value).toBe(false);
    expect(list.items.value).toEqual(["item 1.1", "item 1.2"]);
    const last = notify.notifications.value.at(-1);
    expect(last?.kind).toBe("danger");
    expect(last?.text).toBe("Не удалось загрузить: boom");
  });
});
