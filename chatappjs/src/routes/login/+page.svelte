<script lang="ts">
  import "./style.css";
  import { PUBLIC_URL_BACKEND } from "$env/static/public";
  const operacoes = {
    LOGIN: "LOGIN",
    REGISTRO: "REGISTRO",
  } as const;
  type Operacoes = keyof typeof operacoes;

  let usuario = "";
  let senha = "";
  let erro: string | undefined = undefined;
  let mensagem: string | undefined = undefined;
  let operacaoAtual: Operacoes = operacoes.LOGIN;
  function limparInput() {
    usuario = "";
    senha = "";
  }
  function processarDados() {
    if (operacaoAtual === operacoes.LOGIN) logar();
    else if (operacaoAtual === operacoes.REGISTRO) registrar();
  }
  async function logar() {
    await fetch("http://" + PUBLIC_URL_BACKEND + "/user/login", {
      method: "POST",
      body: JSON.stringify({
        usuario,
        senha,
      }),
      headers: { "Content-Type": "application/json" },
    });
    limparInput();
  }
  async function registrar() {
    limparInput();
  }
</script>

<div id="page">
  <div id="container-auth">
    {#if erro}
      <div id="caixa-erro">
        {erro}
      </div>
    {/if}
    {#if operacaoAtual === operacoes.LOGIN}
      <p id="container-auth-title">Entrar</p>
    {:else if operacaoAtual === operacoes.REGISTRO}
      <p id="container-auth-title">Registrar</p>
    {/if}

    <section id="auth-input-holder">
      <label class="input-label">Usuario</label>
      <input
        bind:value={usuario}
        on:keydown={(e) => {
          if (e.key === "Enter") processarDados();
        }}
      />
      <label class="input-label">Senha</label>
      <input
        type="password"
        bind:value={senha}
        on:keydown={(e) => {
          if (e.key === "Enter") processarDados();
        }}
      />
    </section>

    <footer id="container-auth-footer">
      {#if operacaoAtual === operacoes.LOGIN}
        <p class="full-width">
          Não tem uma conta? <button
            class="text-button"
            on:click={() => {
              operacaoAtual = operacoes.REGISTRO;
            }}>Cadastrar.</button
          >
        </p>
        <button
          id="send-button"
          on:click={() => {
            processarDados();
          }}>Entrar</button
        >
      {:else if operacaoAtual === operacoes.REGISTRO}
        <p class="full-width">
          Já está cadastrado? <button
            class="text-button"
            on:click={() => {
              operacaoAtual = operacoes.LOGIN;
            }}>Logar.</button
          >
        </p>
        <button
          id="send-button"
          on:click={() => {
            processarDados();
          }}>Registrar</button
        >
      {/if}
    </footer>
  </div>
</div>
