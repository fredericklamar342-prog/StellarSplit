import type { Group, ApiResponse } from "../types/split-group";

const API_BASE = "/api/groups";

export async function fetchGroups(): Promise<ApiResponse<Group[]>> {
  const res = await fetch(API_BASE);
  return res.json();
}

export async function createGroup(name: string, members: string[]): Promise<ApiResponse<Group>> {
  const res = await fetch(API_BASE, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ name, members }),
  });
  return res.json();
}

export async function addMember(groupId: string, member: string): Promise<ApiResponse<Group>> {
  const res = await fetch(`${API_BASE}/${groupId}/members`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ member }),
  });
  return res.json();
}

export async function startSplit(groupId: string): Promise<ApiResponse<Group>> {
  const res = await fetch(`${API_BASE}/${groupId}/split`, { method: "POST" });
  return res.json();
}
