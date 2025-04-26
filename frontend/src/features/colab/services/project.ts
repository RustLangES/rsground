import { BACKEND_HOST } from "@services";

export async function createProject(owner: string, name: string = "Unnamed"): Promise<string> {
  let res = await fetch(`${BACKEND_HOST}/create/${name}`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${owner}`,
    },
  });

  if (!res.ok) {
    return null
  }

  return (await res.json()).id;
}
