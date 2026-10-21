import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

const LEVELS = [
  ["video", "Con video"],
  ["highlighted", "Evidenziato"],
  ["picture_only", "Solo immagini"],
  ["independent", "Da solo"],
];

function shuffle(arr) {
  const a = [...arr];
  for (let i = a.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [a[i], a[j]] = [a[j], a[i]];
  }
  return a;
}

export default function App() {
  const [task, setTask] = useState(null);
  const [screen, setScreen] = useState("start"); // start | play | end
  const [players, setPlayers] = useState([]);
  const [newName, setNewName] = useState("");
  const [name, setName] = useState("");
  const [level, setLevel] = useState("video");
  const [index, setIndex] = useState(0);
  const [options, setOptions] = useState([]);
  const [errors, setErrors] = useState(0);
  const [startedAt, setStartedAt] = useState(0);
  const [message, setMessage] = useState("");
  const [locked, setLocked] = useState(false);
  const [wrongId, setWrongId] = useState(null);
  const [result, setResult] = useState(null);
  const [confirmDelete, setConfirmDelete] = useState(null);
  const [exportMsg, setExportMsg] = useState("");

  useEffect(() => {
    invoke("get_task").then(setTask).catch(console.error);
    invoke("list_players").then(setPlayers).catch(console.error);
  }, []);

  useEffect(() => {
    if (screen !== "play" || !task) return;
    const step = task.steps[index];
    const others = task.steps.filter((s) => s.id !== step.id);
    setOptions(shuffle([step, ...shuffle(others).slice(0, 2)]));
    setMessage("");
    setLocked(false);
    setWrongId(null);
  }, [screen, task, index]);

  function selectPlayer(p) {
    setName(p);
    invoke("start_player", { name: p }).then(setLevel);
  }

  async function addPlayer() {
    if (!newName.trim()) return;
    const list = await invoke("add_player", { name: newName });
    setPlayers(list);
    setNewName("");
  }

  async function exportExcel() {
    try {
      const path = await invoke("export_excel");
      setExportMsg("Salvato: " + path);
    } catch (e) {
      setExportMsg(String(e));
    }
  }

  async function deletePlayer() {
    const list = await invoke("delete_player", { name: confirmDelete });
    setPlayers(list);
    if (name === confirmDelete) setName("");
    setConfirmDelete(null);
  }

  function begin(chosenLevel) {
    setLevel(chosenLevel);
    setIndex(0);
    setErrors(0);
    setStartedAt(Date.now());
    setScreen("play");
  }

  async function finish() {
    const seconds = Math.round((Date.now() - startedAt) / 1000);
    const res = await invoke("finish_lap", { player: name, level, errors, seconds });
    setResult({ errors, seconds, ...res });
    setScreen("end");
  }

  function choose(option) {
    if (locked) return;
    const step = task.steps[index];
    if (option.id === step.id) {
      setLocked(true);
      setMessage("Bravo!");
      const last = index + 1 >= task.steps.length;
      setTimeout(() => (last ? finish() : setIndex((i) => i + 1)), 900);
    } else {
      setErrors((e) => e + 1);
      setMessage("Riprova");
      setWrongId(option.id);
      setTimeout(() => setWrongId(null), 450);
    }
  }

  if (!task) return <main className="app"><p>Caricamento...</p></main>;

  /* ---------- старт ---------- */
  if (screen === "start") {
    return (
      <main className="app">
        <h1>Lavarsi le mani</h1>
        <h2>Chi sei?</h2>
        <div className="players">
          {players.map((p) => (
            <button
              key={p}
              className={"btn player" + (name === p ? " selected" : "")}
              onClick={() => selectPlayer(p)}
              onContextMenu={(e) => {
                e.preventDefault();
                setConfirmDelete(p);
              }}
            >
              {p}
            </button>
          ))}
        </div>

        <h2>Livello</h2>
        <div className="levels">
          {LEVELS.map(([id, label]) => (
            <button
              key={id}
              className={"btn level" + (level === id ? " selected" : "")}
              onClick={() => setLevel(id)}
            >
              {label}
            </button>
          ))}
        </div>

        <button className="btn primary" disabled={!name} onClick={() => begin(level)}>
          Via!
        </button>

        <details className="staff">
          <summary>Operatore</summary>
          <input
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            placeholder="Nome"
          />
          <button className="btn" onClick={addPlayer}>Aggiungi persona</button>
          
          <button className="btn" onClick={exportExcel}>Esporta in Excel</button>
          {exportMsg && <p className="export-msg">{exportMsg}</p>}

          <p>Tasto destro su un nome per eliminarlo.</p>
        </details>

        {confirmDelete && (
          <div className="overlay">
            <div className="dialog">
              <h2>Eliminare {confirmDelete}?</h2>
              <p>Verrà cancellata anche tutta la cronologia.</p>
              <div className="row">
                <button className="btn" onClick={() => setConfirmDelete(null)}>No</button>
                <button className="btn danger" onClick={deletePlayer}>Sì, elimina</button>
              </div>
            </div>
          </div>
        )}
      </main>
    );
  }

  /* ---------- конец круга ---------- */
  if (screen === "end") {
    return (
      <main className="app end">
        <h1>Bravo! 🎉</h1>
        <p className="stats">
          Errori: {result.errors} · Tempo: {result.seconds} s
        </p>
        {result.advanced && <p className="badge">Livello superiore! ⭐</p>}
        {!result.advanced && result.clean_streak > 0 && (
          <p className="stats">
            Giro pulito {result.clean_streak} / {result.needed}
          </p>
        )}
        <div className="end-buttons">
          <button className="btn primary" onClick={() => begin(result.next)}>
            Ancora
          </button>
          <button
            className="btn"
            onClick={() => {
              setLevel(result.next);
              setScreen("start");
            }}
          >
            Cambia livello
          </button>
        </div>
      </main>
    );
  }

  /* ---------- игра ---------- */
  const step = task.steps[index];
  const showBigPicture = level === "video";
  const showHighlight = level === "video" || level === "highlighted";
  const showButtonPictures = level === "picture_only";

  return (
    <main className="app">
      <div className="progress">
        {task.steps.map((s, i) => (
          <span
            key={s.id}
            className={"dot" + (i < index ? " done" : i === index ? " current" : "")}
          />
        ))}
      </div>

      <h2 className="question">Cosa si fa adesso?</h2>

      {showBigPicture && <img className="big-picture" src={"/" + step.image} alt="" />}

      <div className={"options" + (locked ? " locked" : "")}>
        {options.map((opt) => {
          const cls =
            "btn option" +
            (showHighlight && opt.id === step.id ? " hint" : "") +
            (wrongId === opt.id ? " wrong" : "");
          return (
            <button key={opt.id} className={cls} onClick={() => choose(opt)}>
              {showButtonPictures && <img src={"/" + opt.image} alt="" />}
              {opt.title}
            </button>
          );
        })}
      </div>

      <p className={"message " + (message === "Bravo!" ? "ok" : "retry")}>{message}</p>
    </main>
  );
}