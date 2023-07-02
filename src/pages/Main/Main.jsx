import { useForm } from "react-hook-form"

import styles from "./Main.module.css"
import { invoke } from "@tauri-apps/api"
import { listen } from "@tauri-apps/api/event"
import { useEffect, useState } from "react"

const Main = () => {
  const {
    register,
    handleSubmit,
    watch,
    formState: {
      errors
    },
    reset
  } = useForm({
    defaultValues: {
      username: ""
    }
  })

  const [username, setUsername] = useState("")
  const [profiles, setProfiles] = useState([])
  const [isDownloading, setIsDownloading] = useState(false)

  const [currentProfile, setCurrentProfile] = useState({})

  const profileId = watch("profile")

  useEffect(() => {
    for (let index = 0; index < profiles.length; index++) {
      const element = profiles[index];
      if (element.id == profileId) {
        setCurrentProfile(element)
      }
    }
  },[profileId])
  
  useEffect(() => {
    invoke("get_config").then((resp) => {
      setProfiles(resp.profiles)
      setUsername(resp.username)
      reset({ username: username })
    })
  }, [])


  const onSubmit = async (data) => {
    if (currentProfile.is_downloaded) {
      invoke("launch", {
        username: data.username,
        version: currentProfile.version
      })
    } else {
      const unlisten = await listen('downloading', (event) => {
        console.log(event.payload);
      });
      invoke("download_version", {id: currentProfile.version})
    }
  }

  return (
    <form onSubmit={handleSubmit(onSubmit)} className={styles.form}>
      <input type="text" {...register("username", {
        required: true,
        minLength: 3,
        maxLength: 16,
        pattern: /^[a-zA-Z0-9_]{3,16}$/mg
      })}
      className={styles.input}
      />

      {errors.username && <div>
        <span>Requirements:</span>
          <ul>
            <li>
              Needs to consist of 3-16 characters
            </li>
            <li>
              No spaces
            </li>
          </ul>

          <span>Allowed characters:</span>
          <ul>
            <li>
              A-Z (upper and lower case)
            </li>
            <li>
              0-9
            </li>
            <li>
              The only allowed special character is _ (underscore)
            </li>
          </ul>
      </div>}

      <div className={styles.select}>
        {/* TODO: Add a customizable select */}
        <span>Select profile</span>
        {
          profiles.map((option) => (
            <label key={option.id}>
              <input {...register("profile", {
                required: true
              })} key={option.id} value={option.id} type="radio" />
              {option.name}
            </label>
          ))
        }
      </div>
      {errors.profile && <p>You must select a profile to launch</p>}

      <button type="submit" className={styles.button}>{currentProfile.is_downloaded ? "Launch" : "Download"}</button>
    </form>
  )
}

export default Main