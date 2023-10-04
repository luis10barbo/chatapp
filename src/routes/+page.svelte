<script lang="ts">
  import "./style.css";
</script>

<section id="chats">
  <header id="chats-header" class="section-header">
    <p>Conversas</p>
  </header>
  <section id="chats-holder">
    <div id="chat-search-holder">
      <input placeholder="Pesquisar uma conversa..." />
      <button>Pesquisar</button>
    </div>

    <button class="chat-card">
      <header class="chat-card-header">
        <p class="chat-card-name">Titulo</p>
      </header>
      <footer class="chat-card-footer">
        <p class="chat-card-time">10:45</p>
        <p class="chat-card-status">
          <span class="chat-card-status-name">Luis</span>
          <span class="chat-card-status-msg">teste teste</span>
        </p>
      </footer>
    </button>
  </section>
  <footer id="chats-footer" class="section-footer">
    <button>Contatos</button>
    <button>Perfil</button>
    <button>Sair</button>
  </footer>
</section>
<section id="curr-chat">
  <header id="curr-chat-header" class="section-header">
    <img id="img-curr-chat" />
    <div id="curr-chat-info">
      <p>Grupo Atual</p>
      <p class="chat-status" />
    </div>
    <p id="curr-chat-online-holder">
      <span id="curr-chat-online-count">0</span> Online
    </p>
  </header>
  <div id="curr-chat-messages-holder">
    <div class="msg-card">
      <header class="msg-card-header">
        <p class="msg-card-name">Luis</p>
      </header>
      <footer class="msg-card-footer">
        <p class="msg-card-time">10:45</p>
        <p class="msg-card-status">
          <span class="msg-card-msg">Alguem aqui no chat</span>
        </p>
      </footer>
    </div>
  </div>
  <footer id="curr-chat-footer" class="section-footer">
    <input id="send-message" placeholder="Escreva uma mensagem..." />
    <button id="send-message-button">Enviar</button>
  </footer>
</section>
