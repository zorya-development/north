import type { BrowserContext } from "@playwright/test";

const BASE_URL = process.env.BASE_URL || "http://localhost:5000";

interface CreateTaskParams {
  title: string;
  parent_id?: number;
  project_id?: number;
  sort_key?: string;
  start_at?: string;
  due_date?: string;
}

interface UpdateTaskParams {
  title?: string;
  sequential_limit?: number;
  completed_at?: string | null;
  sort_key?: string;
  start_at?: string | null;
  due_date?: string | null;
  body?: string | null;
}

interface Task {
  id: number;
  title: string;
  parent_id: number | null;
  project_id: number | null;
  sort_key: string;
  sequential_limit: number;
  completed_at: string | null;
  reviewed_at: string | null;
  start_at: string | null;
  due_date: string | null;
  [key: string]: unknown;
}

interface CreateProjectParams {
  title: string;
  color?: string;
}

interface UpdateProjectParams {
  title?: string;
  status?: string;
  color?: string;
}

interface Project {
  id: number;
  title: string;
  status: string;
  color: string;
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

  // --- Tasks ---

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

  async reviewTask(id: number): Promise<Task> {
    const res = await this.request("PATCH", `/tasks/${id}/review`);
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

  // --- Projects ---

  async createProject(params: CreateProjectParams): Promise<Project> {
    const res = await this.request("POST", "/projects", params);
    return res.json();
  }

  async updateProject(
    id: number,
    params: UpdateProjectParams,
  ): Promise<Project> {
    const res = await this.request("PATCH", `/projects/${id}`, params);
    return res.json();
  }

  async listProjects(params?: {
    status?: string;
  }): Promise<Project[]> {
    let path = "/projects";
    if (params?.status) {
      path += `?status=${params.status}`;
    }
    const res = await this.request("GET", path);
    return res.json();
  }

  async deleteProject(id: number): Promise<void> {
    const token = await this.getToken();
    await fetch(`${BASE_URL}/api/projects/${id}`, {
      method: "DELETE",
      headers: { Authorization: `Bearer ${token}` },
    });
  }

  async deleteAllProjects(): Promise<void> {
    const active = await this.listProjects();
    const archived = await this.listProjects({ status: "archived" });
    for (const project of [...active, ...archived]) {
      await this.deleteProject(project.id);
    }
  }
}
