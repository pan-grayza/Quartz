interface LinkedPath {
    name: string
    path: string
}
interface LocalNetwork {
    name: string
    port: number
    linkedPaths: LinkedPath[] // Vec<LinkedPath> becomes LinkedPath[]
}
interface InternetNetwork {
    name: string
    address: string
}
interface DarkWebNetwork {
    name: string
    address: string
}
type Network =
    | { LocalNetwork: LocalNetwork }
    | { InternetNetwork: InternetNetwork }
    | { DarkWebNetwork: DarkWebNetwork }

type ServerMode = 'LocalHost' | 'Internet' | 'DarkWeb'
