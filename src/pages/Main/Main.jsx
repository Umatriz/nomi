import { useForm } from "react-hook-form"

import styles from "./Main.module.css"
import { invoke } from "@tauri-apps/api"
import { useEffect, useState } from "react"

const Main = () => {
  const [username, setUsername] = useState("")
  const [profiles, setProfiles] = useState([])
  
  useEffect(() => {
    invoke("get_config").then((resp) => {
      setProfiles(resp.profiles)
      setUsername(resp.username)
    })
  }, [])

  const {
    register,
    handleSubmit,
    formState: {
      errors
    },
  } = useForm()

  const onSubmit = (data) => {
    console.log(data)
    console.log(username)
  }

  return (
    <form onSubmit={handleSubmit(onSubmit)} className={styles.form}>
      <input defaultValue={username} type="text" {...register("username", {
        required: true,
        minLength: 3,
        maxLength: 16,
        pattern: /^[a-zA-Z0-9_]{3,16}$/mg
      })}
      className={styles.input}
      />

      {errors?.username && <div>
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
      {errors?.profile && <p>You must select a profile to launch</p>}

      <input type="submit" className={styles.button} value="Launch" />
    </form>
  )
}

export default Main