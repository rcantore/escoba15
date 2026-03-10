# Escoba del 15

Un motor de juego para la [Escoba del 15](https://es.wikipedia.org/wiki/Escoba_del_15), el clasico juego de cartas jugado en toda Espana y Latinoamerica. Hecho en Rust con una IA que usa simulaciones de Monte Carlo para jugar — sin reglas hardcodeadas, sin arboles de `if/else`, solo miles de partidas simuladas por jugada.

> **[English version](README.en.md)**

## Que es la Escoba del 15?

Dos jugadores se turnan para jugar cartas de su mano. El objetivo es capturar cartas de la mesa encontrando combinaciones que **sumen 15** con la carta que jugas. Si limpiaste toda la mesa en una captura, eso es una **escoba**. Al final de cada ronda se suman puntos:

| Categoria | Punto |
|-----------|-------|
| Mas cartas capturadas | 1 |
| Mas oros capturados | 1 |
| El 7 de Oros (*siete de velo*) | 1 |
| Mas sietes capturados | 1 |
| Cada escoba | 1 |

El primero en llegar a **15 puntos** entre rondas gana la partida.

El mazo es la clasica **baraja espanola de 40 cartas**: 4 palos (Oros, Copas, Espadas, Bastos), numeros del 1 al 7 y del 10 al 12 (sin 8 ni 9).

## Estructura del Proyecto

```
escoba15/
├── engine/          # Motor del juego (Rust)
│   ├── src/
│   │   ├── card.rs      # Modelo de carta (palo, numero, valores)
│   │   ├── deck.rs      # Baraja espanola de 40 cartas
│   │   ├── player.rs    # Estado del jugador (mano, capturas, escobas)
│   │   ├── game.rs      # Logica del juego, busqueda de combinaciones, turnos
│   │   ├── scoring.rs   # Puntuacion por ronda y deteccion de fin de partida
│   │   ├── ai.rs        # IA basada en MCTS
│   │   ├── lang.rs      # Soporte multilenguaje (Espanol/Ingles)
│   │   ├── wasm.rs      # Bridge WebAssembly (opcional)
│   │   └── lib.rs       # API publica
│   └── tests/
│       └── engine_tests.rs  # 49 tests de integracion
├── cli/             # Interfaz de terminal
│   └── src/
│       ├── main.rs      # Loop de juego interactivo
│       └── strings.rs   # Textos localizados
└── Cargo.toml       # Configuracion del workspace
```

## Como Funciona la IA

La IA no tiene una lista de "si sale esta carta, juega esta otra". En su lugar, usa **Information Set Monte Carlo Tree Search (ISMCTS)** — una tecnica de investigacion en IA para juegos con informacion oculta.

Esto es lo que pasa cada vez que la IA tiene que jugar:

### 1. El Problema: Informacion Oculta

La escoba es un juego de **informacion imperfecta**. La IA puede ver su propia mano y la mesa, pero no sabe:

- Que cartas tiene el oponente
- En que orden esta el mazo

Esto significa que no puede calcular la jugada "perfecta" como en el ajedrez. Tiene que razonar con incertidumbre.

### 2. Determinizacion: Imaginando Mundos Posibles

Para cada simulacion, la IA toma todas las cartas que **no puede ver** (mano del oponente + mazo restante) y las **mezcla al azar**. Despues las reparte: la cantidad correcta vuelve a la mano del oponente, el resto forma el mazo.

Esto crea una version posible de la realidad — un escenario de "que pasaria si las cartas estuvieran *asi*?". Se llama **determinizacion**.

```
Lo que la IA sabe:            Lo que la IA imagina:
┌─────────────────┐           ┌─────────────────┐
│ Mi mano: 3,7,R  │           │ Mi mano: 3,7,R  │  (igual)
│ Mesa: 5,S       │           │ Mesa: 5,S       │  (igual)
│ Su mano: ???    │     →     │ Su mano: 2,6,C  │  (suposicion al azar)
│ Mazo: ?????????  │           │ Mazo: 1,4,A,B...│  (mezclado)
└─────────────────┘           └─────────────────┘
```

### 3. Playout: Simular Hasta el Final

En cada mundo imaginado, la IA prueba una jugada especifica (por ejemplo, "jugar el 7 para capturar el 5+3 de la mesa"), y despues **ambos jugadores juegan al azar** hasta que termina la ronda. Esto se llama **playout aleatorio**. Es rapido porque no hay que pensar — ambos lados simplemente eligen jugadas legales al azar.

### 4. Puntuacion: Quien Gano?

Cuando termina el playout, la IA cuenta los puntos: quien capturo mas cartas? Mas oros? El siete de velo? Escobas? Registra el resultado como victoria, derrota o empate.

### 5. Repetir Miles de Veces

La IA hace esto con todas sus jugadas posibles. Para cada jugada, corre muchas simulaciones, y la jugada con el **mayor porcentaje de victorias** es la que elige.

La **dificultad** controla cuantas simulaciones se corren:

| Dificultad | Simulaciones | Tiempo |
|-----------|-------------|--------|
| Facil     | 100         | Instantaneo |
| Medio     | 1.000       | Rapido |
| Dificil   | 10.000      | ~1 segundo |

Mas simulaciones = mejor confianza estadistica = juego mas fuerte. La IA en dificil genuinamente considera el valor estrategico de cada jugada a traves de miles de futuros aleatorios.

### Por Que Este Enfoque?

- **Sin heuristicas manuales.** La IA descubre que es bueno simulando, no porque alguien programo "preferi los oros".
- **Escala con computo.** Queres una IA mas fuerte? Solo aumenta la cantidad de simulaciones.
- **Maneja la incertidumbre naturalmente.** La determinizacion es una tecnica probada para juegos de informacion imperfecta.
- **Funciona de verdad.** En dificil, la IA toma decisiones estrategicas sorprendentemente buenas — prioriza escobas, junta oros y protege cartas clave.

## Busqueda de Combinaciones

Otro detalle interesante: encontrar que cartas de la mesa suman 15 con tu carta. El motor usa **enumeracion de subconjuntos por bitmask** — genera todos los 2^n subconjuntos de las cartas de la mesa usando operaciones de bits y verifica cuales dan la suma objetivo. Con un maximo de ~10 cartas en la mesa, son como mucho 1024 subconjuntos — instantaneo.

```rust
for mask in 1..(1u32 << n) {
    let subset_sum = (0..n)
        .filter(|bit| mask & (1 << bit) != 0)
        .map(|bit| table[bit].value())
        .sum();
    if hand_value + subset_sum == 15 {
        // Encontramos una captura valida!
    }
}
```

## Como Empezar

### Requisitos

- [Rust](https://rustup.rs/) (1.70+)

### Compilar y Testear

```bash
# Correr todos los tests (69 tests: 20 unitarios + 49 de integracion)
cargo test

# Compilar el motor
cargo build --release

# Jugar en la terminal
cargo run --release -p escoba15-cli
```

### Compilar para WebAssembly

El motor se compila a WASM para UIs en el navegador:

```bash
# Instalar wasm-pack
cargo install wasm-pack

# Compilar el paquete WASM
cd engine
wasm-pack build --target web --features wasm
```

Esto genera un directorio `pkg/` con `.wasm` + bindings de JS listos para importar desde cualquier framework web.

## Usar el Motor como Libreria

```rust
use escoba15_engine::*;

// Crear una partida nueva
let mut game = Game::new("Alice", "Bob");
game.deal_round();

// Obtener jugadas validas para el jugador actual
let plays = game.valid_plays();

// Jugar una carta (capturar cartas de la mesa en indices 0 y 2)
let result = game.play_card(0, Some(vec![0, 2]));

// O tirar una carta (sin captura)
let result = game.play_card(1, None);

// Avanzar al siguiente turno
game.next_turn();

// Pedirle a la IA una jugada
let ai_move = suggest_play(&game, Difficulty::Hard);

// Calcular puntos al final de la ronda
let scores = calculate_score(
    &game.players[0].captured,
    game.players[0].escobas,
    &game.players[1].captured,
);
```

## Multilenguaje

El motor soporta espanol e ingles nativamente:

```rust
use escoba15_engine::{Card, Suit, Lang};

let card = Card::new(Suit::Oros, 12);
card.localized_name(Lang::Es); // "Rey de Oros"
card.localized_name(Lang::En); // "King of Coins"
```

## Cobertura de Tests

69 tests cubriendo:

- Creacion de cartas, valores y casos borde
- Operaciones del mazo (mezclar, robar, vacio)
- Busqueda de combinaciones (mesa vacia, sin match, match unico, multiples, combos de tres cartas)
- Jugadas (captura, descarte, deteccion de escoba, jugadas invalidas, reparto de ronda)
- Puntuacion (cartas, oros, siete de velo, sietes, escobas, totales)
- Deteccion de fin de partida (umbral, empates, mayor puntaje gana)
- IA (jugadas validas, preferencia de captura, integridad de determinizacion, finalizacion de playout, escalado de dificultad)

```bash
cargo test
# running 69 tests ... test result: ok. 69 passed
```

## Licencia

MIT
