import { invoke } from '@tauri-apps/api'
import { useEffect, useState } from 'react'

import Main from "./components/Main/Main"

function App() {
  // const [manifest, setManifest] = useState([])

  // useEffect(() => {
  //   invoke("get_manifest").then((res) => {
  //     setManifest(res)
  //   })
  // }, [])

  // useEffect(() => {
  //   console.log(manifest)
  // }, [manifest])

  return (
    <>
      <Main />
    </>
  )
}

export default App
