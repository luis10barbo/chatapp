import { PUBLIC_URL_BACKEND } from "$env/static/public";

export async function postJson(url: string, corpo: Object) {
  return await fetch(url, {
    method: "POST",
    body: JSON.stringify(corpo),
    credentials: "include",
    headers: { "Content-Type": "application/json" },
  });
}
export async function getJson(url: string) {
  return await fetch(url, {
    method: "GET",
    credentials: "include",
    headers: { "Content-Type": "application/json" },
  });
}

export async function requestPerfil() {
  return getJson(
    window.location.protocol + "//" + PUBLIC_URL_BACKEND + "/user/me"
  );
}
