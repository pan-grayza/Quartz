interface LinkedPath {
    name: string
    path: string
}
interface BaseNetwork {
    name: string
    linked_paths: LinkedPath[]
}

interface LocalNetwork extends BaseNetwork {
    port: number
}

interface InternetNetwork extends BaseNetwork {
    address: string
}

interface DarkWebNetwork extends BaseNetwork {
    address: string
}

// Union of network types
type Network = LocalNetwork | InternetNetwork | DarkWebNetwork

type ServerMode = 'LocalHost' | 'Internet' | 'DarkWeb'
