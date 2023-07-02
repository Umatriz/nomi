import { invoke } from "@tauri-apps/api"
import { useForm } from "react-hook-form"

import styles from "./Profile.module.css"
import { useEffect, useState } from "react"

const Profile = () => {
  const [manifest, setManifest] = useState([])

  useEffect(() => {
    invoke("get_manifest").then((resp) => {
      setManifest(resp)
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
  }

  return (
    <>
      <form onSubmit={handleSubmit(onSubmit)} className={styles.form}>
        <input type="text" placeholder="Profile name" {...register("name", {
          required: true,
          // FIXME
          pattern: /^[^\s].*[^\s]$/
        })} className={styles.input} />
        {errors.name && <p>{errors.name.message}</p>}
        <div className={styles.select}>
          <span>Select profile</span>
          {
            manifest.map((option) => (
              <label key={option.id}>
                <input {...register("version", {
                  required: true
                })} key={option.id} value={option.id} type="radio" />
                {option.id}
              </label>
            ))
          }
        </div>
        <input type="submit" className={styles.button} value="Create" />
      </form>
    </>
  )
}

export default Profile