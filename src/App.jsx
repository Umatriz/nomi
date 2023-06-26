import { invoke } from '@tauri-apps/api'
import { useEffect, useState } from 'react'

function App() {
  const [manifest, setManifest] = useState([])

  useEffect(() => {
    invoke("get_manifest").then((res) => {
      setManifest(res)
      console.log(manifest)
      console.log(res)
    })
  }, [])

  const buttonTest = () => {
    invoke("get_manifest").then((resp) => {
      setManifest(resp)
      console.log(manifest)
    })
  }

  return (
    <>
      <div>
        <button onClick={buttonTest}>R</button>
      </div>
    </>
  )
}

export default App
