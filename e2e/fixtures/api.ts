import type { BrowserContext } from "@playwright/test";

const BASE_URL = process.env.BASE_URL || "http://localhost:5000";

interface CreateTaskParams {
  title: string;
  parent_id?: number;
  project_id?: number;
  sort_key?: string;
}

interface UpdateTaskParams {
  title?: string;
  sequential_limit?: number;
  completed_at?: string | null;
  sort_key?: string;
}

interface Task {
  id: number;
  title: string;
  parent_id: number | null;
  project_id: number | null;
  sort_key: string;
  sequential_limit: number;
  completed_at: string | null;
  [key: string]: unknown;
}

/**
 * REST API helper that uses the auth cookie from a Playwright browser context.
 */
export class ApiHelper {
  private token: string | undefined;

  constructor(private context: BrowserContext) {}

  private async getToken(): Promise<string> {
    if (this.token) return this.token;
    const cookies = await this.context.cookies();
    const tokenCookie = cookies.find((c) => c.name === "token");
    if (!tokenCookie) {
      throw new Error("No auth token cookie found — login first");
    }
    this.token = tokenCookie.value;
    return this.token;
  }

  private async request(
    method: string,
    path: string,
    body?: unknown,
  ): Promise<Response> {
    const token = await this.getToken();
    const res = await fetch(`${BASE_URL}/api${path}`, {
      method,
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
      },
      body: body !== undefined ? JSON.stringify(body) : undefined,
    });
    if (!res.ok && res.status !== 204) {
      const text = await res.text();
      throw new Error(
        `API ${method} ${path} failed (${res.status}): ${text}`,
      );
    }
    return res;
  }

  async createTask(params: CreateTaskParams): Promise<Task> {
    const res = await this.request("POST", "/tasks", params);
    return res.json();
  }

  async updateTask(id: number, params: UpdateTaskParams): Promise<Task> {
    const res = await this.request("PATCH", `/tasks/${id}`, params);
    return res.json();
  }

  async deleteTask(id: number): Promise<void> {
    await this.request("DELETE", `/tasks/${id}`);
  }

  async listTasks(): Promise<Task[]> {
    const res = await this.request("GET", "/tasks");
    return res.json();
  }

  async deleteAllTasks(): Promise<void> {
    const tasks = await this.listTasks();
    // Delete children first (tasks with parent_id), then roots
    const children = tasks.filter((t) => t.parent_id !== null);
    const roots = tasks.filter((t) => t.parent_id === null);
    for (const task of [...children, ...roots]) {
      await this.deleteTask(task.id);
    }
  }
}
