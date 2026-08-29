import { Logo } from "./components/logo";
import { useGames } from "./hooks/use-games";

export function App() {
  const { data: games, isLoading, error } = useGames();

  return (
    <div className="h-svh flex items-center justify-center">
      <Logo className="w-32" />
      <div>
        {isLoading && <p>Loading games...</p>}
        {error && <p>Error: {String(error)}</p>}
        {games && (
          <ul>
            {games.map((game) => (
              <li key={game.id}>{game.title}</li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}

