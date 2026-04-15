import { Environment, Grid, Html, Sparkles } from "@react-three/drei";
import { Canvas, useFrame, useThree } from "@react-three/fiber";
import { Bloom, EffectComposer, Vignette } from "@react-three/postprocessing";
import { useMemo, useRef } from "react";
import * as THREE from "three";
import type { PlayerView } from "../hooks/useCipherSocket";

const HALF = 400;

function LookAtArena() {
  const { camera } = useThree();
  useFrame(() => {
    camera.lookAt(0, 0, 0);
  });
  return null;
}

function ArenaFloor() {
  return (
    <mesh rotation-x={-Math.PI / 2} receiveShadow position={[0, 0, 0]}>
      <planeGeometry args={[800, 800]} />
      <meshStandardMaterial
        color="#0d1526"
        metalness={0.28}
        roughness={0.62}
        emissive="#1a2744"
        emissiveIntensity={0.12}
      />
    </mesh>
  );
}

function ArenaRim() {
  const mat = useMemo(
    () =>
      new THREE.MeshStandardMaterial({
        color: "#1a2744",
        metalness: 0.45,
        roughness: 0.42,
        emissive: "#2b8ef0",
        emissiveIntensity: 0.12,
      }),
    [],
  );
  const h = 28;
  const t = 10;
  return (
    <group>
      <mesh position={[0, h / 2, -HALF - t / 2]} castShadow material={mat}>
        <boxGeometry args={[820, h, t]} />
      </mesh>
      <mesh position={[0, h / 2, HALF + t / 2]} castShadow material={mat}>
        <boxGeometry args={[820, h, t]} />
      </mesh>
      <mesh position={[-HALF - t / 2, h / 2, 0]} castShadow material={mat}>
        <boxGeometry args={[t, h, 820]} />
      </mesh>
      <mesh position={[HALF + t / 2, h / 2, 0]} castShadow material={mat}>
        <boxGeometry args={[t, h, 820]} />
      </mesh>
    </group>
  );
}

function PlayerOrb({ p, isSelf }: { p: PlayerView; isSelf: boolean }) {
  const x = p.x - HALF;
  const z = p.y - HALF;
  const group = useRef<THREE.Group>(null);
  useFrame((state) => {
    if (!group.current || !isSelf) return;
    const t = state.clock.elapsedTime;
    group.current.position.y = 18 + Math.sin(t * 6) * 1.2;
  });

  if (!p.alive) return null;

  return (
    <group ref={group} position={[x, isSelf ? 18 : 18, z]}>
      <mesh castShadow>
        <sphereGeometry args={[30, 40, 40]} />
        <meshStandardMaterial
          color={p.color}
          emissive={p.color}
          emissiveIntensity={isSelf ? 1.35 : 0.72}
          metalness={0.22}
          roughness={0.32}
        />
      </mesh>
      <mesh rotation-x={-Math.PI / 2} position={[0, -16, 0]}>
        <ringGeometry args={[34, 42, 48]} />
        <meshBasicMaterial
          color={p.color}
          transparent
          opacity={0.45}
          depthWrite={false}
        />
      </mesh>
      <Html center distanceFactor={88} zIndexRange={[100, 0]} style={{ pointerEvents: "none" }}>
        <div className="rounded border border-[#2b8ef0]/40 bg-[#0a0f1e]/80 px-2 py-0.5 font-mono text-[10px] font-bold uppercase tracking-widest text-[#f0f4ff] shadow-[0_0_20px_rgba(43,142,240,0.35)] backdrop-blur-sm">
          {p.name}
        </div>
      </Html>
    </group>
  );
}

function Effects() {
  return (
    <EffectComposer disableNormalPass>
      <Bloom luminanceThreshold={0.25} mipmapBlur intensity={1.15} radius={0.5} />
      <Vignette eskil={false} offset={0.12} darkness={0.55} />
    </EffectComposer>
  );
}

type ArenaSceneProps = {
  players: PlayerView[];
  myId: string | null;
};

export function ArenaScene({ players, myId }: ArenaSceneProps) {
  return (
    <Canvas
      shadows
      dpr={[1, 2]}
      gl={{ antialias: true, alpha: false }}
      camera={{ position: [540, 460, 540], fov: 42, near: 1, far: 8000 }}
      className="h-full w-full"
    >
      <color attach="background" args={["#060912"]} />
      <fog attach="fog" args={["#0a0f1e", 900, 2600]} />

      <ambientLight intensity={0.35} />
      <directionalLight
        castShadow
        position={[220, 520, 180]}
        intensity={1.15}
        shadow-mapSize-width={2048}
        shadow-mapSize-height={2048}
        shadow-camera-far={2400}
        shadow-camera-left={-700}
        shadow-camera-right={700}
        shadow-camera-top={700}
        shadow-camera-bottom={-700}
      />
      <pointLight position={[-200, 120, -200]} intensity={0.8} color="#00e5ff" />
      <pointLight position={[260, 80, 260]} intensity={0.55} color="#ff5f9e" />

      <LookAtArena />
      <ArenaFloor />
      <ArenaRim />

      <Grid
        infiniteGrid
        fadeDistance={1200}
        fadeStrength={1}
        sectionColor="#2b8ef0"
        cellColor="#1a2744"
        sectionThickness={1.2}
        cellThickness={0.6}
        sectionSize={80}
        cellSize={20}
        position={[0, 0.05, 0]}
      />

      <Sparkles
        count={120}
        scale={[720, 40, 720]}
        position={[0, 80, 0]}
        size={4}
        speed={0.35}
        color="#2b8ef0"
        opacity={0.55}
      />

      <Environment preset="night" environmentIntensity={0.35} />

      {players.map((p) => (
        <PlayerOrb key={p.id} p={p} isSelf={p.id === myId} />
      ))}

      <Effects />
    </Canvas>
  );
}
