<script lang="ts">
  import { requestUser, type Usuario } from "./+page.svelte";
  import type { Chat } from "./ContainerChatSelector.svelte";

  export let chat: Chat;
  export let click: (e: MouseEvent) => void;

  let user: Usuario | undefined = undefined;
  async function getUsuario() {
    if (!chat.last_message) return;
    user = await requestUser(chat.last_message.user_id);
  }
  getUsuario();
</script>

<button class="chat-card" on:click={click}>
  <header class="chat-card-header">
    <p class="chat-card-name">{chat.chat_name}</p>
  </header>
  <footer class="chat-card-footer">
    {#if chat.last_message}
      <p class="chat-card-time">{chat.last_message.date_created}</p>
      <p class="chat-card-status">
        <span class="chat-card-status-name">{user?.user_nick}</span>
        <span class="chat-card-status-msg">{chat.last_message.message}</span>
      </p>
    {/if}
  </footer>
</button>
